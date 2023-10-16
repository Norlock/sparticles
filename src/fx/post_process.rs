use super::blur::BlurData;
use super::{color_processing::ColorProcessingUniform, Bloom, ColorProcessing};
use crate::init::AppSettings;
use crate::model::{GfxState, State};
use crate::traits::*;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{ShaderType, UniformBuffer};

pub struct PostProcessState {
    pub post_fx: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,

    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
    uniform: OffsetUniform,
    offset_buffer: wgpu::Buffer,
}

#[derive(ShaderType, Clone)]
pub struct OffsetUniform {
    offset: i32,
    view_width: f32,
    view_height: f32,
}

pub struct CreateFxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
}

impl OffsetUniform {
    fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            offset: config.fx_offset() as i32,
            view_width: config.width as f32,
            view_height: config.height as f32,
        }
    }

    fn buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        let config = &gfx_state.surface_config;
        let queue = &gfx_state.queue;

        self.uniform = OffsetUniform::new(config);
        queue.write_buffer(&self.offset_buffer, 0, &self.uniform.buffer_content());

        self.fx_state.resize(config.fx_dimensions(), gfx_state);
    }

    pub fn compute(state: &mut State, encoder: &mut wgpu::CommandEncoder) {
        let pp = &mut state.post_process;
        let input = pp.fx_state.bind_group(1);

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
        });

        c_pass.set_pipeline(&pp.initialize_pipeline);
        c_pass.set_bind_group(0, input, &[]);
        //c_pass.set_bind_group(1, pp.frame_state.output(), &[]);
        c_pass.dispatch_workgroups(pp.fx_state.count_x, pp.fx_state.count_y, 1);

        //for (i, pfx) in pp.post_fx.iter().filter(|fx| fx.enabled()).enumerate() {
        //let input = pp.fx_state.bind_group(i);
        //pfx.compute(input, &mut c_pass);
        //}
    }

    pub fn render(
        state: &mut State,
        output_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let clipped_primitives = GfxState::draw_gui(state, encoder);

        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post process render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let pp = &mut state.post_process;
        r_pass.set_pipeline(&pp.finalize_pipeline);
        r_pass.set_bind_group(0, pp.fx_state.bind_group(0), &[]);
        //r_pass.set_bind_group(1, pp.frame_state.output(), &[]);
        r_pass.draw(0..3, 0..1);

        state.gfx_state.renderer.render(
            &mut r_pass,
            &clipped_primitives,
            &state.gfx_state.screen_descriptor,
        );
    }

    pub fn export(pp: &PostProcessState) {
        //Persistence::write_to_file(to_export, ExportType::PostFx);
    }

    pub fn new(gfx_state: &GfxState, app_settings: &impl AppSettings) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let initialize_shader = device.create_shader("fx/initialize.wgsl", "Init post fx");
        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let uniform = OffsetUniform::new(config);
        let buffer_content = uniform.buffer_content();

        let offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offset"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fx_state = FxState::new(FxStateOptions {
            tex_dimensions: config.fx_dimensions(),
            gfx_state,
        });

        let fx_group_layout = &fx_state.bind_group_layout;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[fx_group_layout],
            push_constant_ranges: &[],
        });

        let initialize_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init pipeline"),
                layout: Some(&pipeline_layout),
                module: &initialize_shader,
                entry_point: "init",
            });

        let finalize_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Finalize pipeline"),
            layout: Some(&pipeline_layout),
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

        let post_fx = app_settings.add_post_fx(&CreateFxOptions {
            fx_state: &fx_state,
            gfx_state,
        });

        Self {
            fx_state,
            post_fx,
            initialize_pipeline,
            finalize_pipeline,
            offset_buffer,
            uniform,
        }
    }

    pub fn create_fx_options<'a>(&'a self, gfx_state: &'a GfxState) -> CreateFxOptions {
        CreateFxOptions {
            gfx_state,
            fx_state: &self.fx_state,
        }
    }

    pub fn add_default_fx(&mut self, gfx_state: &GfxState) {
        // TODO remove default fx
        let options = self.create_fx_options(gfx_state);

        let bloom = Bloom::new(&options, BlurData::default());
        let col_cor = ColorProcessing::new(&options, ColorProcessingUniform::default());

        self.post_fx.push(Box::new(bloom));
        self.post_fx.push(Box::new(col_cor));
    }

    pub fn import_fx(&mut self, gfx_state: &GfxState) {
        // TODO
    }
}

pub struct FxState {
    bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub count_x: u32,
    pub count_y: u32,
    pub fx_view_1: wgpu::TextureView,
    pub fx_view_2: wgpu::TextureView,
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
}

struct FxStateOptions<'a> {
    /// For debugging purposes
    tex_dimensions: Dimensions,
    gfx_state: &'a GfxState,
}

pub type Dimensions = [u32; 2];

impl FxState {
    pub fn bind_group(&self, idx: usize) -> &wgpu::BindGroup {
        &self.bind_groups[idx % 2]
    }

    fn get_dispatch_counts(dimensions: Dimensions) -> [u32; 2] {
        let count_x = (dimensions[0] as f32 / WORK_GROUP_SIZE[0]).ceil() as u32;
        let count_y = (dimensions[1] as f32 / WORK_GROUP_SIZE[1]).ceil() as u32;

        [count_x, count_y]
    }

    pub fn resize(&mut self, dimensions: Dimensions, gfx_state: &GfxState) {
        *self = Self::new(FxStateOptions {
            gfx_state,
            tex_dimensions: dimensions,
        });
    }

    fn new(options: FxStateOptions) -> Self {
        let FxStateOptions {
            tex_dimensions,
            gfx_state,
        } = options;

        let device = &gfx_state.device;
        let frame_view = gfx_state.create_frame_view();
        let depth_view = gfx_state.create_depth_view();

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
                // Frame
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

        let mut bind_groups = Vec::new();
        let fx_view_1 = gfx_state.create_fx_view();
        let fx_view_2 = gfx_state.create_fx_view();
        let fx_views = vec![&fx_view_1, &fx_view_2];

        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
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
                ],
            });

            bind_groups.push(bg);
        }

        let counts = Self::get_dispatch_counts(tex_dimensions);

        Self {
            bind_groups,
            bind_group_layout,
            count_x: counts[0],
            count_y: counts[1],
            fx_view_1,
            fx_view_2,
            depth_view,
            frame_view,
        }
    }
}

impl FxDimensions for wgpu::SurfaceConfiguration {
    fn fx_dimensions(&self) -> Dimensions {
        let expand = self.fx_offset() * 2;

        [self.width + expand, self.height + expand]
    }

    fn fx_offset(&self) -> u32 {
        0 //(self.width / 60).max(32)
    }
}
