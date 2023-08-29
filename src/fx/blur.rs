use super::PostProcessState;
use super::WORK_GROUP_SIZE;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{ShaderType, UniformBuffer};
use std::num::NonZeroU64;

pub struct Blur {
    blur_pipelines: Vec<wgpu::ComputePipeline>,
    split_pipeline: wgpu::ComputePipeline,

    uniform_bind_group: wgpu::BindGroup,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bloom_uniform: BlurUniform,
    pub bloom_buffer: wgpu::Buffer,

    res: BloomResources,
    passes: usize,
}

#[derive(Copy, Clone, Debug, ShaderType)]
pub struct BlurUniform {
    /// 0.10 - 0.15 is reasonable
    pub brightness_threshold: f32,
    /// 2.2 - 2.6 is reasonable
    pub gamma: f32,
    /// Kernel size (8 default) too high or too low slows down performance
    /// Lower is more precise
    pub kernel_size: u32,

    // How far should the blur reach (in relation with kernel size)
    pub radius: u32,
    //pub depth_add: f32,
    //pub depth_mul: f32,
}

impl BlurUniform {
    pub fn new() -> Self {
        Self {
            brightness_threshold: 0.2,
            gamma: 2.2,
            kernel_size: 16,
            radius: 16,
        }
    }

    pub fn create_buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

impl PostProcessFx for Blur {
    fn compute<'a>(&'a self, input: Vec<&'a wgpu::BindGroup>, c_pass: &mut wgpu::ComputePass<'a>) {
        let BloomResources {
            bind_groups,
            count_x,
            count_y,
        } = &self.res;

        // Splits parts to fx tex
        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, input[0], &[]);
        c_pass.set_bind_group(1, &bind_groups[1], &[]);
        c_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
        c_pass.dispatch_workgroups(*count_x, *count_y, 1);

        // Smoothen downscaled texture
        for i in 0..self.passes {
            let nr = i % 2;

            c_pass.set_pipeline(&self.blur_pipelines[nr]);
            c_pass.set_bind_group(0, input[0], &[]);
            c_pass.set_bind_group(1, &bind_groups[nr], &[]);
            c_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
            c_pass.dispatch_workgroups(*count_x, *count_y, 1);
        }
    }

    fn resize(&mut self, gfx_state: &GfxState, _dispatch_xy: &[u32; 2]) {
        self.res = BloomResources::new(
            gfx_state,
            &self.bind_group_layout,
            self.bloom_uniform.kernel_size,
        );
    }

    fn enabled(&self) -> bool {
        true
    }
}

impl Blur {
    pub fn output(&self) -> &wgpu::BindGroup {
        &self.res.bind_groups[self.passes % 2]
    }

    pub fn new(gfx_state: &GfxState, pp: &PostProcessState, shader_entry: &str) -> Self {
        let device = &gfx_state.device;

        let bloom_uniform = BlurUniform::new();
        let buffer_content = bloom_uniform.create_buffer_content();
        let min_binding_size = NonZeroU64::new(buffer_content.len() as u64);

        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let bloom_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bloom uniform"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let uni_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom uniform layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom uniform bind group"),
            layout: &uni_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bloom_buffer.as_entire_binding(),
            }],
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom textures layout"),
            entries: &[
                // FX Write
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: PostProcessState::TEXTURE_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
                // FX Read
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[
                &pp.bind_group_layout,
                &bind_group_layout,
                &uni_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let new_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gaussian blur pipeline"),
                layout: Some(&pipeline_layout),
                module: &blur_shader,
                entry_point,
            })
        };

        let blur_pipelines = vec![new_pipeline("blur_x"), new_pipeline("blur_y")];
        let split_pipeline = new_pipeline(shader_entry);

        let res = BloomResources::new(gfx_state, &bind_group_layout, bloom_uniform.kernel_size);

        let passes = 8;

        Self {
            blur_pipelines,
            bind_group_layout,
            uniform_bind_group,
            bloom_buffer,
            bloom_uniform,
            res,
            split_pipeline,
            passes,
        }
    }
}

pub struct BloomResources {
    bind_groups: Vec<wgpu::BindGroup>,
    count_x: u32,
    count_y: u32,
}

impl BloomResources {
    pub fn new(
        gfx_state: &GfxState,
        bind_group_layout: &wgpu::BindGroupLayout,
        kernel_size: u32,
    ) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let tex_width = (config.width as f32 / kernel_size as f32).ceil();
        let tex_height = (config.height as f32 / kernel_size as f32).ceil();

        let mut fx_views = Vec::new();

        for _ in 0..2 {
            let fx_view = gfx_state.create_fx_view(tex_width as u32, tex_height as u32);
            fx_views.push(fx_view);
        }

        let fx_count_x = (tex_width / WORK_GROUP_SIZE[0]).ceil() as u32;
        let fx_count_y = (tex_height / WORK_GROUP_SIZE[1]).ceil() as u32;

        // Create ping pong bind group
        let create_bind_group = |src_view, dst_view| -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(dst_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(src_view),
                    },
                ],
            })
        };

        let b_grp_1 = create_bind_group(&fx_views[0], &fx_views[1]);
        let b_grp_2 = create_bind_group(&fx_views[1], &fx_views[0]);

        let bind_groups = vec![b_grp_1, b_grp_2];

        Self {
            bind_groups,
            count_x: fx_count_x,
            count_y: fx_count_y,
        }
    }
}
