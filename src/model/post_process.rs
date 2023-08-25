use std::num::NonZeroU64;

use crate::traits::*;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{ShaderType, UniformBuffer};

use super::GfxState;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const WORK_GROUP_SIZE: [u8; 2] = [8, 8];

pub struct Bloom {
    pub compute_pipelines: Vec<wgpu::ComputePipeline>,
    pub split_channel_pipeline: wgpu::ComputePipeline,

    pub tex_bind_group_layout: wgpu::BindGroupLayout,

    pub render_pipeline: wgpu::RenderPipeline,

    pub uni_bind_group: wgpu::BindGroup,
    pub uni_bind_group_layout: wgpu::BindGroupLayout,

    pub bloom_uniform: BloomUniform,
    pub bloom_buffer: wgpu::Buffer,
    pub res: BloomResources,
}

#[derive(Copy, Clone, Debug, ShaderType)]
pub struct BloomUniform {
    /// 0.10 - 0.15 is reasonable
    pub brightness_threshold: f32,
    /// 2.2 - 2.6 is reasonable
    pub gamma: f32,
    /// Weight applied on offset
    pub weight_1: f32,
    pub weight_2: f32,
    pub weight_3: f32,
    pub weight_4: f32,
    pub weight_5: f32,
}

impl Default for BloomUniform {
    fn default() -> Self {
        Self {
            brightness_threshold: 0.5,
            gamma: 2.2,
            weight_1: 0.227027,
            weight_2: 0.1945946,
            weight_3: 0.1216216,
            weight_4: 0.054054,
            weight_5: 0.016216,
        }
    }
}

impl BloomUniform {
    pub fn create_buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

impl Bloom {
    pub fn compute_fx<'a>(&'a self, c_pass: &mut wgpu::ComputePass<'a>) {
        // Splits everything that needs to be blurred
        c_pass.set_pipeline(&self.split_channel_pipeline);
        c_pass.set_bind_group(0, &self.res.tex_bind_groups[1], &[]);
        c_pass.set_bind_group(1, &self.uni_bind_group, &[]);
        c_pass.dispatch_workgroups(self.res.work_group_count_x, self.res.work_group_count_y, 1);

        // 10 passes
        for i in 0..10 {
            let pipe_nr = (i % 5).min(1);

            c_pass.set_pipeline(&self.compute_pipelines[pipe_nr]);
            c_pass.set_bind_group(0, &self.res.tex_bind_groups[i % 2], &[]);
            c_pass.set_bind_group(1, &self.uni_bind_group, &[]);
            c_pass.dispatch_workgroups(self.res.work_group_count_x, self.res.work_group_count_y, 1);
        }
    }

    pub fn render_fx<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        r_pass.set_pipeline(&self.render_pipeline);
        r_pass.set_bind_group(0, &self.res.tex_bind_groups[0], &[]); // or 1
        r_pass.set_bind_group(1, &self.uni_bind_group, &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.res = BloomResources::new(gfx_state, &self.tex_bind_group_layout);
    }
}

impl GfxState {
    pub fn create_post_process(&self) -> Bloom {
        let device = &self.device;

        let bloom_uniform = BloomUniform::default();
        let buffer_content = bloom_uniform.create_buffer_content();
        let min_binding_size = NonZeroU64::new(buffer_content.len() as u64);

        let blur_shader = device.create_shader("gaussian_blur.wgsl", "Gaussian blur");
        let blend_shader = device.create_shader("bloom.wgsl", "Additivie blending shader");

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
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size,
                    },
                    count: None,
                }],
            });

        let uni_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom uniform bind group"),
            layout: &uni_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bloom_buffer.as_entire_binding(),
            }],
        });

        let tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom textures layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: TEXTURE_FORMAT,
                            access: wgpu::StorageTextureAccess::WriteOnly,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post processing layout"),
            bind_group_layouts: &[&tex_bind_group_layout, &uni_bind_group_layout],
            push_constant_ranges: &[],
        });

        let create_c_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gaussian blur pipeline"),
                layout: Some(&compute_layout),
                module: &blur_shader,
                entry_point,
            })
        };

        let compute_pipelines = vec![create_c_pipeline("blur_x"), create_c_pipeline("blur_y")];
        let split_channel_pipeline = create_c_pipeline("split");

        // Render
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend render"),
            bind_group_layouts: &[&tex_bind_group_layout, &uni_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blend pipeline"),
            layout: Some(&render_layout),
            vertex: wgpu::VertexState {
                module: &blend_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &blend_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let res = BloomResources::new(&self, &tex_bind_group_layout);

        Bloom {
            compute_pipelines,
            tex_bind_group_layout,
            uni_bind_group,
            uni_bind_group_layout,
            render_pipeline,
            bloom_buffer,
            bloom_uniform,
            res,
            split_channel_pipeline,
        }
    }

    fn create_fx_texture(&self) -> wgpu::Texture {
        let config = &self.surface_config;

        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        })
    }

    fn create_frame_texture(&self) -> wgpu::Texture {
        let config = &self.surface_config;

        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        })
    }
}

pub struct BloomResources {
    fx_texs: Vec<wgpu::Texture>,
    fx_tex_views: Vec<wgpu::TextureView>,
    tex_bind_groups: Vec<wgpu::BindGroup>,

    pub frame_tex_view: wgpu::TextureView,

    work_group_count_x: u32,
    work_group_count_y: u32,
}

impl BloomResources {
    pub fn new(
        gfx_state: &GfxState,
        fx_layout: &wgpu::BindGroupLayout, // fx
    ) -> Self {
        // TODO create another struct for code reuse
        let frame_tex = gfx_state.create_frame_texture();
        let frame_tex_view = frame_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut fx_texs = Vec::new();
        let mut fx_tex_views = Vec::new();

        for _ in 0..2 {
            let tex = gfx_state.create_fx_texture();
            let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

            fx_texs.push(tex);
            fx_tex_views.push(tex_view);
        }

        let work_group_count_x =
            (fx_texs[0].width() as f32 / WORK_GROUP_SIZE[0] as f32).ceil() as u32;
        let work_group_count_y =
            (fx_texs[0].height() as f32 / WORK_GROUP_SIZE[1] as f32).ceil() as u32;

        let device = &gfx_state.device;

        // Create ping pong bind group
        let create_fx_bind_group = |src_tex_view, dst_tex_view| -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &fx_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(dst_tex_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(src_tex_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&frame_tex_view),
                    },
                ],
            })
        };

        let b_grp_1 = create_fx_bind_group(&fx_tex_views[0], &fx_tex_views[1]);
        let b_grp_2 = create_fx_bind_group(&fx_tex_views[1], &fx_tex_views[0]);

        let tex_bind_groups = vec![b_grp_1, b_grp_2];

        Self {
            fx_texs,
            fx_tex_views,
            tex_bind_groups,
            frame_tex_view,
            work_group_count_x,
            work_group_count_y,
        }
    }
}
