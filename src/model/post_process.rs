use crate::traits::CustomShader;
use egui_wgpu::wgpu;

use super::GfxState;

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const WORK_GROUP_SIZE: [u8; 2] = [8, 8];

pub struct PostProcess {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub render_bind_group: wgpu::BindGroup,
    pub render_bind_group_layout: wgpu::BindGroupLayout,
    pub fx_tex: wgpu::Texture,
    pub frame_tex: wgpu::Texture,
    pub fx_tex_view: wgpu::TextureView,
    pub frame_tex_view: wgpu::TextureView,
    pub work_group_count_x: u32,
    pub work_group_count_y: u32,
}

impl PostProcess {
    pub fn compute_fx<'a>(&'a self, c_pass: &mut wgpu::ComputePass<'a>) {
        c_pass.set_pipeline(&self.compute_pipeline);
        c_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        c_pass.dispatch_workgroups(self.work_group_count_x, self.work_group_count_y, 1);
    }

    pub fn render_fx<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        r_pass.set_pipeline(&self.render_pipeline);
        r_pass.set_bind_group(0, &self.render_bind_group, &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.frame_tex = gfx_state.create_frame_texture();
        self.frame_tex_view = self
            .frame_tex
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.fx_tex = gfx_state.create_fx_texture();
        self.fx_tex_view = self
            .fx_tex
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.work_group_count_x =
            (self.fx_tex.width() as f32 / WORK_GROUP_SIZE[0] as f32).ceil() as u32;
        self.work_group_count_y =
            (self.fx_tex.height() as f32 / WORK_GROUP_SIZE[1] as f32).ceil() as u32;

        let device = &gfx_state.device;

        self.compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.fx_tex_view),
            }],
        });

        self.render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.frame_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.fx_tex_view),
                },
            ],
        });
    }
}

impl GfxState {
    pub fn create_post_process(&self) -> PostProcess {
        let device = &self.device;

        let shader = self
            .device
            .create_shader("post_process.wgsl", "Post process");

        let frame_tex = self.create_frame_texture();
        let frame_tex_view = frame_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let fx_tex = self.create_fx_texture();
        let fx_tex_view = fx_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Create texture
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Post processing"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: TEXTURE_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                }],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&fx_tex_view),
            }],
        });

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post processing layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Post process pipeline"),
            layout: Some(&compute_layout),
            module: &shader,
            entry_point: "main",
        });

        // Render
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Post processing blend"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend render"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&frame_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fx_tex_view),
                },
            ],
        });

        let blend_shader = device.create_shader("blend.wgsl", "Blend shader");

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

        let work_group_count_x = (fx_tex.width() as f32 / WORK_GROUP_SIZE[0] as f32).ceil() as u32;
        let work_group_count_y = (fx_tex.height() as f32 / WORK_GROUP_SIZE[1] as f32).ceil() as u32;

        PostProcess {
            compute_pipeline,
            compute_bind_group,
            compute_bind_group_layout,
            render_pipeline,
            render_bind_group,
            render_bind_group_layout,
            fx_tex,
            fx_tex_view,
            frame_tex,
            frame_tex_view,
            work_group_count_x,
            work_group_count_y,
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
