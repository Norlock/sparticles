use crate::init::AppSettings;
use crate::model::{GfxState, State};
use crate::traits::*;
use crate::util::{
    CommonBuffer, DynamicExport, ExportType, ListAction, Persistence, UniformContext,
};
use egui_wgpu::wgpu;
use encase::ShaderType;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

pub struct PostProcessState {
    pub effects: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,
    pub io_uniform: FxIOUniform,

    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,

    pub io_ctx: UniformContext,
}

pub struct PingPongState {
    fx_idx: usize,
}

impl PingPongState {
    fn new() -> Self {
        Self { fx_idx: 0 }
    }

    fn idx(&self) -> usize {
        self.fx_idx
    }

    pub fn swap(&mut self) {
        self.fx_idx = (self.fx_idx + 1) % 2;
    }
}

#[derive(ShaderType, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct FxIOUniform {
    pub in_idx: u32,
    pub out_idx: u32,

    pub in_downscale: u32,
    pub out_downscale: u32,

    pub in_size_x: u32,
    pub in_size_y: u32,

    pub out_size_x: u32,
    pub out_size_y: u32,
}

pub struct FxIO {
    pub in_idx: u32,
    pub out_idx: u32,

    pub in_downscale: u32,
    pub out_downscale: u32,

    pub in_size: Vec2,
    pub out_size: Vec2,
}

pub struct FxIOUniformOptions {
    pub in_idx: u32,
    pub out_idx: u32,
    pub in_downscale: f32,
    pub out_downscale: f32,
}

impl FxIOUniform {
    pub fn create_downscale_list(
        list: &mut Vec<FxIO>,
        fx_size: &glam::Vec2,
        downscale_count: i32,
        in_downscale: u32,
        io_idx: u32,
    ) -> Vec<Self> {
        let out_downscale = in_downscale * 2;
        let out_size;

        if let Some(last) = list.iter().last() {
            let in_size = last.out_size;
            out_size = (in_size / 2.).ceil();

            list.push(FxIO {
                in_idx: io_idx,
                out_idx: io_idx + 1,
                in_size,
                out_size,
                in_downscale,
                out_downscale,
            });
        } else {
            let in_size = *fx_size;
            out_size = (in_size / 2.).ceil();

            list.push(FxIO {
                in_idx: io_idx,
                out_idx: io_idx + 1,
                in_size,
                out_size,
                in_downscale,
                out_downscale,
            });
        }

        if 0 <= downscale_count - 1 {
            Self::create_downscale_list(
                list,
                fx_size,
                downscale_count - 1,
                out_downscale,
                io_idx + 1,
            )
        } else {
            list.iter()
                .map(|io| Self {
                    in_idx: io.in_idx,
                    out_idx: io.out_idx,
                    in_downscale: io.in_downscale,
                    out_downscale: io.out_downscale,
                    in_size_x: io.in_size.x as u32,
                    in_size_y: io.in_size.y as u32,
                    out_size_x: io.out_size.x as u32,
                    out_size_y: io.out_size.y as u32,
                })
                .collect()
        }
    }

    pub fn symetric_unscaled(fx_state: &FxState, io_idx: u32) -> Self {
        Self::create(
            fx_state,
            FxIOUniformOptions {
                in_idx: io_idx,
                out_idx: io_idx,
                in_downscale: 1.,
                out_downscale: 1.,
            },
        )
    }

    pub fn asymetric_unscaled(fx_state: &FxState, in_idx: u32, out_idx: u32) -> Self {
        Self::create(
            fx_state,
            FxIOUniformOptions {
                in_idx,
                out_idx,
                in_downscale: 1.,
                out_downscale: 1.,
            },
        )
    }

    pub fn reverse_list(list: &Vec<Self>) -> Vec<Self> {
        let mut result = Vec::new();

        for last in list.iter().rev() {
            result.push(Self {
                in_idx: last.out_idx,
                out_idx: last.in_idx,
                in_size_x: last.out_size_x,
                in_size_y: last.out_size_y,
                out_size_x: last.in_size_x,
                out_size_y: last.in_size_y,
                in_downscale: last.out_downscale,
                out_downscale: last.in_downscale,
            });
        }

        result
    }

    pub fn create(fx_state: &FxState, options: FxIOUniformOptions) -> Self {
        let FxIOUniformOptions {
            in_idx,
            out_idx,
            in_downscale,
            out_downscale,
        } = options;

        let in_size = (fx_state.tex_size / in_downscale).ceil();
        let out_size = (fx_state.tex_size / out_downscale).ceil();

        Self {
            in_idx,
            out_idx,
            in_size_x: in_size.x as u32,
            in_size_y: in_size.y as u32,
            out_size_x: out_size.x as u32,
            out_size_y: out_size.y as u32,
            in_downscale: 1,
            out_downscale: 1,
        }
    }

    pub fn create_content(&self) -> Vec<u8> {
        CommonBuffer::uniform_content(self)
    }

    pub fn zero(fx_state: &FxState) -> Self {
        let tex_size = &fx_state.tex_size;
        let size_x = tex_size.x as u32;
        let size_y = tex_size.y as u32;

        Self {
            in_idx: 0,
            out_idx: 0,
            in_size_x: size_x,
            in_size_y: size_y,
            out_size_x: size_x,
            out_size_y: size_y,
            in_downscale: 1,
            out_downscale: 1,
        }
    }

    pub fn resize(&mut self, io_ctx: &UniformContext, options: &CreateFxOptions) {
        let fx_state = &options.fx_state;
        let tex_size = &fx_state.tex_size;

        let in_size = (*tex_size / self.in_downscale as f32).ceil();
        let out_size = (*tex_size / self.out_downscale as f32).ceil();

        *self = Self {
            in_idx: self.in_idx,
            out_idx: self.out_idx,
            in_size_x: in_size.x as u32,
            in_size_y: in_size.y as u32,
            out_size_x: out_size.x as u32,
            out_size_y: out_size.y as u32,
            in_downscale: self.in_downscale,
            out_downscale: self.out_downscale,
        };

        let queue = &options.gfx_state.queue;
        let contents = CommonBuffer::uniform_content(self);

        queue.write_buffer(&io_ctx.buf, 0, &contents);
    }
}

pub struct CreateFxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
}

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.fx_state = FxState::new(gfx_state);

        let options = CreateFxOptions {
            fx_state: &self.fx_state,
            gfx_state,
        };

        self.io_uniform.resize(&self.io_ctx, &options);

        for fx in self.effects.iter_mut() {
            fx.resize(&options);
        }
    }

    pub fn update(state: &mut State) {
        let effects = &mut state.post_process.effects;

        for fx in effects.iter_mut() {
            fx.update(&state.gfx_state);
        }

        ListAction::update_list(effects);
    }

    pub fn frame_view(&self) -> &wgpu::TextureView {
        &self.fx_state.frame_view
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.fx_state.depth_view
    }

    pub fn compute(state: &mut State, encoder: &mut wgpu::CommandEncoder) -> PingPongState {
        let profiler = &mut state.gfx_state.profiler;
        let pp = &mut state.post_process;
        let fx_state = &mut pp.fx_state;
        let device = &state.gfx_state.device;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
            timestamp_writes: None,
        });

        let mut ping_pong = PingPongState::new();

        profiler.begin_scope("Post fx compute", &mut c_pass, &device);
        profiler.begin_scope("Init", &mut c_pass, &device);
        c_pass.set_pipeline(&pp.initialize_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &pp.io_ctx.bg, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);
        profiler.end_scope(&mut c_pass).unwrap();

        ping_pong.swap();

        for fx in pp.effects.iter().filter(|fx| fx.enabled()) {
            fx.compute(&mut ping_pong, &fx_state, &mut state.gfx_state, &mut c_pass);
        }

        state.gfx_state.profiler.end_scope(&mut c_pass).unwrap();

        ping_pong
    }

    pub fn render(
        state: &mut State,
        output_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        ping_pong: PingPongState,
    ) {
        let clipped_primitives = GfxState::draw_gui(state, encoder);
        let profiler = &mut state.gfx_state.profiler;
        let device = &state.gfx_state.device;

        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post process render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        profiler.begin_scope("Post fx render", &mut r_pass, &device);
        let pp = &mut state.post_process;
        let gui = &state.gui;

        r_pass.set_pipeline(&pp.finalize_pipeline);

        r_pass.set_bind_group(0, &pp.fx_state.r_bg, &[]);
        r_pass.set_bind_group(1, &pp.io_ctx.bg, &[]);
        r_pass.draw(0..4, 0..1);

        state.gfx_state.renderer.render(
            &mut r_pass,
            &clipped_primitives,
            &state.gfx_state.screen_descriptor,
        );

        profiler.end_scope(&mut r_pass).unwrap();
    }

    pub fn new(gfx_state: &GfxState, app_settings: &impl AppSettings) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let initialize_shader = device.create_shader("fx/initialize.wgsl", "Init post fx");
        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let fx_state = FxState::new(gfx_state);

        let io_uniform = FxIOUniform::zero(&fx_state);
        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "IO");

        let c_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[&fx_state.bg_layout, &io_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let r_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[&fx_state.r_bg_layout, &io_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let initialize_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init pipeline"),
                layout: Some(&c_pipeline_layout),
                module: &initialize_shader,
                entry_point: "init",
            });

        let finalize_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Finalize pipeline"),
            layout: Some(&r_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &finalize_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
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

        let effects = app_settings.add_post_fx(&CreateFxOptions {
            fx_state: &fx_state,
            gfx_state,
        });

        Self {
            fx_state,
            effects,
            initialize_pipeline,
            finalize_pipeline,

            io_uniform,
            io_ctx,
        }
    }

    pub fn import_fx(
        &mut self,
        gfx_state: &GfxState,
        registered_effects: &Vec<Box<dyn RegisterPostFx>>,
        to_export: Vec<DynamicExport>,
    ) {
        let options = CreateFxOptions {
            gfx_state,
            fx_state: &self.fx_state,
        };

        for item in to_export {
            for reg in registered_effects {
                if item.tag == reg.tag() {
                    self.effects.push(reg.import(&options, item.data));
                    break;
                }
            }
        }
    }

    pub fn export(pp: &PostProcessState) {
        let mut to_export = Vec::new();

        for fx in pp.effects.iter() {
            to_export.push(fx.export());
        }

        Persistence::write_to_file(to_export, ExportType::PostFx);
    }
}

pub struct FxState {
    pub bg: wgpu::BindGroup,
    pub bg_layout: wgpu::BindGroupLayout,

    r_bg: wgpu::BindGroup,
    r_bg_layout: wgpu::BindGroupLayout,

    pub count_x: u32,
    pub count_y: u32,

    pub tex_size: glam::Vec2,
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
}

const WORK_GROUP_SIZE: f32 = 8.;

impl FxState {
    pub fn count_in(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let res = (self.tex_size / io_uniform.in_downscale as f32 / WORK_GROUP_SIZE).ceil();

        //(res.x as u32, res.y as u32)
        (self.count_x, self.count_y)
    }

    pub fn count_out(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let res = (self.tex_size / io_uniform.out_downscale as f32 / WORK_GROUP_SIZE).ceil();

        //(res.x as u32, res.y as u32)
        (self.count_x, self.count_y)
    }

    fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let frame_view = gfx_state.create_frame_view();
        let depth_view = gfx_state.create_depth_view();

        let array_count = 16;

        let c_layout_entries = [
            // Fx read + write
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    view_dimension: wgpu::TextureViewDimension::D2,
                    format: PostProcessState::TEXTURE_FORMAT,
                    access: wgpu::StorageTextureAccess::ReadWrite,
                },
                count: NonZeroU32::new(array_count),
            },
            // Frame
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
            // Depth
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
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        let r_layout_entries = [
            // Fx read + write
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    multisampled: false,
                },
                count: NonZeroU32::new(array_count),
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post process layout"),
            entries: &c_layout_entries,
        });

        let r_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post process render layout"),
            entries: &r_layout_entries,
        });

        let mut tex_views = Vec::new();

        for _ in 0..array_count {
            tex_views.push(gfx_state.create_fx_view());
        }

        let tex_refs: Vec<&wgpu::TextureView> = tex_views.iter().collect();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fx compute bindgroup"),
            layout: &bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&tex_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&frame_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let r_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fx render bindgroup"),
            layout: &r_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&tex_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let (x, y) = gfx_state.dimensions();
        println!("x: {}, y: {}", x, y);

        let count_x = (x / WORK_GROUP_SIZE).ceil() as u32;
        let count_y = (y / WORK_GROUP_SIZE).ceil() as u32;

        Self {
            bg,
            bg_layout,
            r_bg,
            r_bg_layout,
            tex_size: Vec2::new(x, y),
            count_x,
            count_y,
            depth_view,
            frame_view,
        }
    }
}
