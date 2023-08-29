use std::cell::RefCell;

use crate::traits::*;
use egui_wgpu::wgpu;

use crate::model::GfxState;

use super::{Blend, Bloom};

pub struct PostProcessState {
    pub res: PostProcessResources,
    pub post_fx: Vec<Box<dyn PostProcessFx>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
}

pub struct PostProcessResources {
    fx_bind_groups: Vec<wgpu::BindGroup>,
    active: RefCell<usize>,
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
    pub count_x: u32,
    pub count_y: u32,
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.res = PostProcessResources::new(gfx_state, &self.bind_group_layout);
        let dispatch_xy = self.res.dispatch_xy();

        for pfx in self.post_fx.iter_mut() {
            pfx.resize(&gfx_state, &dispatch_xy);
        }
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
        });

        c_pass.set_pipeline(&self.initialize_pipeline);
        c_pass.set_bind_group(0, &self.res.fx_bind_groups[0], &[]);
        c_pass.dispatch_workgroups(self.res.count_x, self.res.count_y, 1);

        for pfx in self.post_fx.iter() {
            if pfx.enabled() {
                pfx.compute(vec![&self.res.fx_bind_groups[1]], &mut c_pass);
            }
        }
    }

    pub fn render<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        r_pass.set_pipeline(&self.finalize_pipeline);
        r_pass.set_bind_group(0, &self.res.fx_bind_groups[0], &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn create_fx_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post process layout"),
            entries: &[
                // Post fx write
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: Self::TEXTURE_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
                // Post fx read
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
                // Frame
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
                // Depth
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let initialize_shader = device.create_shader("fx/initialize.wgsl", "Init post fx");
        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let bind_group_layout = PostProcessState::create_fx_layout(&device);
        let res = PostProcessResources::new(gfx_state, &bind_group_layout);

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Init layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let initialize_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init pipeline"),
                layout: Some(&compute_layout),
                module: &initialize_shader,
                entry_point: "init",
            });

        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx render"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let finalize_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Finalize pipeline"),
            layout: Some(&render_layout),
            vertex: wgpu::VertexState {
                module: &finalize_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &finalize_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let mut pp = Self {
            res,
            post_fx: vec![],
            bind_group_layout,
            initialize_pipeline,
            finalize_pipeline,
        };

        pp.add_fx(gfx_state);

        return pp;
    }

    fn add_fx(&mut self, gfx_state: &GfxState) {
        let bloom = Bloom::new(gfx_state, &self);

        self.post_fx.push(Box::new(bloom));
    }
}

impl PostProcessResources {
    pub fn dispatch_xy(&self) -> [u32; 2] {
        [self.count_x, self.count_y]
    }

    pub fn new(gfx_state: &GfxState, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let frame_view = gfx_state.create_frame_tex().into_view();
        let depth_view = gfx_state.create_depth_view();
        let fx1_view = gfx_state.create_fx_view(config.width, config.height);
        let fx2_view = gfx_state.create_fx_view(config.width, config.height);
        let fx_views = [&fx1_view, &fx2_view];

        let mut fx_bind_groups = Vec::new();

        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(fx_views[i % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(fx_views[(i + 1) % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&depth_view),
                    },
                ],
            });

            fx_bind_groups.push(bg);
        }

        let count_x = (config.width as f32 / WORK_GROUP_SIZE[0]).ceil() as u32;
        let count_y = (config.height as f32 / WORK_GROUP_SIZE[1]).ceil() as u32;

        Self {
            frame_view,
            depth_view,
            fx_bind_groups,
            count_x,
            count_y,
            active: RefCell::new(0),
        }
    }

    pub fn swap(&self) {
        let idx = (*self.active.borrow() + 1) % 2;
        *self.active.borrow_mut() = idx;
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.fx_bind_groups[*self.active.borrow()]
    }
}
