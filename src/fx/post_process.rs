use crate::traits::*;
use egui_wgpu::wgpu;

use crate::model::GfxState;

use super::{blend::BlendCompute, Blend, BlendType, Bloom};

pub struct PostProcessState {
    pub frame_state: FrameState,
    fx_state: FxState,
    post_fx: Vec<Box<dyn PostFxChain>>,
    frame_group_layout: wgpu::BindGroupLayout,
    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
    blend: Blend,
}

pub struct FrameState {
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

pub struct FxChainOutput<'a> {
    pub blend: BlendType,
    pub bind_group: &'a wgpu::BindGroup,
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    fn render_output(&self) -> &wgpu::BindGroup {
        let nr = self.post_fx.iter().filter(|fx| fx.enabled()).count();

        self.fx_state.bind_group(nr)
    }

    pub fn resize(&mut self, gfx_state: &GfxState) {
        let config = &gfx_state.surface_config;
        self.frame_state = FrameState::new(gfx_state, &self.frame_group_layout);
        self.fx_state.resize(config.width, config.height, gfx_state);

        for pfx in self.post_fx.iter_mut() {
            pfx.resize(&gfx_state);
        }
    }

    pub fn blend<'a>(
        &'a self,
        input: FxChainOutput<'a>,
        output: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let compute = BlendCompute {
            input: input.bind_group,
            output,
            count_x: self.fx_state.count_x,
            count_y: self.fx_state.count_y,
        };

        match input.blend {
            BlendType::ADDITIVE => self.blend.add(compute, c_pass),
            BlendType::BLEND => {
                todo!("todo")
            }
            BlendType::REPLACE => {
                todo!("todo")
            }
        }
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
        });

        c_pass.set_pipeline(&self.initialize_pipeline);
        c_pass.set_bind_group(0, &self.fx_state.bind_group(1), &[]);
        c_pass.set_bind_group(1, &self.frame_state.bind_group, &[]);
        c_pass.dispatch_workgroups(self.fx_state.count_x, self.fx_state.count_y, 1);

        for (i, pfx) in self.post_fx.iter().filter(|fx| fx.enabled()).enumerate() {
            let frame = self.fx_state.bind_group(i);
            let fx = pfx.compute(frame, &mut c_pass);

            self.blend(fx, frame, &mut c_pass);
        }
    }

    pub fn render<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        r_pass.set_pipeline(&self.finalize_pipeline);
        r_pass.set_bind_group(0, self.render_output(), &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn create_fx_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame layout"),
            entries: &[
                // Frame
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
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

        let frame_group_layout = Self::create_fx_layout(&device);
        let frame_state = FrameState::new(gfx_state, &frame_group_layout);

        let fx_state = FxState::new(FxStateOptions {
            label: "Post process start".to_string(),
            tex_width: config.width,
            tex_height: config.height,
            gfx_state,
        });

        let fx_group_layout = &fx_state.bind_group_layout;

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Init layout"),
            bind_group_layouts: &[&fx_group_layout, &frame_group_layout],
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
            bind_group_layouts: &[&fx_group_layout],
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

        let blend = Blend::new(gfx_state, &fx_state);

        Self {
            frame_state,
            fx_state,
            post_fx: vec![],
            frame_group_layout,
            initialize_pipeline,
            finalize_pipeline,
            blend,
        }
        .append_fx(gfx_state)
    }

    fn append_fx(mut self, gfx_state: &GfxState) -> Self {
        let bloom = Bloom::new(gfx_state, &self.frame_state.depth_view);

        self.post_fx.push(Box::new(bloom));

        return self;
    }
}

impl FrameState {
    pub fn new(gfx_state: &GfxState, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let device = &gfx_state.device;

        let frame_view = gfx_state.create_frame_tex().into_view();
        let depth_view = gfx_state.create_depth_view();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&frame_view),
            }],
        });

        Self {
            frame_view,
            depth_view,
            bind_group,
        }
    }
}

pub struct FxState {
    bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub count_x: u32,
    pub count_y: u32,
    pub label: String,
}

pub struct FxStateOptions<'a> {
    /// For debugging purposes
    pub label: String,
    pub tex_width: u32,
    pub tex_height: u32,
    pub gfx_state: &'a GfxState,
}

impl FxState {
    pub fn bind_group(&self, idx: usize) -> &wgpu::BindGroup {
        &self.bind_groups[idx % 2]
    }

    fn create_bind_groups(
        tex_width: u32,
        tex_height: u32,
        layout: &wgpu::BindGroupLayout,
        gfx_state: &GfxState,
    ) -> Vec<wgpu::BindGroup> {
        let device = &gfx_state.device;

        let mut bind_groups = Vec::new();
        let fx_view_1 = gfx_state.create_fx_view(tex_width, tex_height);
        let fx_view_2 = gfx_state.create_fx_view(tex_width, tex_height);
        let fx_views = vec![fx_view_1, fx_view_2];

        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fx_views[i % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&fx_views[(i + 1) % 2]),
                    },
                ],
            });

            bind_groups.push(bg);
        }

        return bind_groups;
    }

    fn get_dispatch_counts(tex_width: u32, tex_height: u32) -> [u32; 2] {
        let count_x = (tex_width as f32 / WORK_GROUP_SIZE[0]).ceil() as u32;
        let count_y = (tex_height as f32 / WORK_GROUP_SIZE[1]).ceil() as u32;

        return [count_x, count_y];
    }

    pub fn resize(&mut self, tex_width: u32, tex_height: u32, gfx_state: &GfxState) {
        self.bind_groups =
            Self::create_bind_groups(tex_width, tex_height, &self.bind_group_layout, gfx_state);

        let counts = Self::get_dispatch_counts(tex_width, tex_height);
        self.count_x = counts[0];
        self.count_y = counts[1];
    }

    pub fn new(options: FxStateOptions) -> Self {
        let FxStateOptions {
            label,
            tex_width,
            tex_height,
            gfx_state,
        } = options;

        let device = &gfx_state.device;

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

        let bind_groups =
            Self::create_bind_groups(tex_width, tex_height, &bind_group_layout, gfx_state);

        let counts = Self::get_dispatch_counts(tex_width, tex_height);

        Self {
            label,
            bind_groups,
            bind_group_layout,
            count_x: counts[0],
            count_y: counts[1],
        }
    }
}

impl FxDimensions for wgpu::SurfaceConfiguration {
    fn fx_dimensions(&self) -> [u32; 2] {
        let expand = self.width / 32 * 2;

        [self.width + expand, self.height + expand]
    }
}
