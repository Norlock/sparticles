use super::{FxIOUniform, FxOptions};
use crate::init::AppSettings;
use crate::model::events::ViewIOEvent;
use crate::model::{GfxState, State};
use crate::traits::*;
use crate::util::{
    CommonBuffer, DynamicExport, ExportType, ListAction, Persistence, UniformContext,
};
use egui_wgpu::wgpu;
use glam::Vec2;
use std::num::NonZeroU32;

pub struct PostProcessState {
    pub effects: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,

    render_pipeline: wgpu::RenderPipeline,

    pub io_uniform: FxIOUniform,
    pub io_ctx: UniformContext,
}

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.fx_state = FxState::new(gfx_state);

        let options = FxOptions {
            fx_state: &self.fx_state,
            gfx_state,
        };

        self.io_uniform.resize(&self.io_ctx.buf, &options);

        for fx in self.effects.iter_mut() {
            fx.resize(&options);
        }
    }

    pub fn update(state: &mut State) {
        let State {
            post_process: pp,
            gfx_state,
            events,
            camera,
            ..
        } = state;

        if let Some(event) = events.get_io_view() {
            let io_uniform = &mut pp.io_uniform;
            match event {
                ViewIOEvent::Add => {
                    if io_uniform.out_idx + 1 < 16 {
                        io_uniform.out_idx += 1;
                    }
                }
                ViewIOEvent::Subtract => {
                    if 0 < io_uniform.out_idx {
                        io_uniform.out_idx -= 1;
                    }
                }
                ViewIOEvent::Idx(val) => {
                    io_uniform.out_idx = val;
                }
            }

            let contents = CommonBuffer::uniform_content(&pp.io_uniform);
            gfx_state.queue.write_buffer(&pp.io_ctx.buf, 0, &contents);
        }

        let effects = &mut pp.effects;

        for fx in effects.iter_mut() {
            fx.update(&state.gfx_state, camera);
        }

        ListAction::update_list(effects);
    }

    pub fn frame_view(&self) -> &wgpu::TextureView {
        &self.fx_state.tex_views[0]
    }

    pub fn split_view(&self) -> &wgpu::TextureView {
        &self.fx_state.tex_views[1]
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.fx_state.depth_view
    }

    pub fn compute(state: &mut State, encoder: &mut wgpu::CommandEncoder) {
        let gfx_state = &mut state.gfx_state;
        let pp = &mut state.post_process;
        let fx_state = &mut pp.fx_state;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
            timestamp_writes: None,
        });

        gfx_state.begin_scope("Post fx compute", &mut c_pass);

        for fx in pp.effects.iter().filter(|fx| fx.enabled()) {
            fx.compute(&fx_state, gfx_state, &mut c_pass);
        }

        gfx_state.end_scope(&mut c_pass);
    }

    pub fn render(
        state: &mut State,
        output_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let clipped_primitives = GfxState::draw_gui(state, encoder);
        let gfx_state = &mut state.gfx_state;

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

        gfx_state.begin_scope("Post fx render", &mut r_pass);
        let pp = &mut state.post_process;

        r_pass.set_pipeline(&pp.render_pipeline);
        r_pass.set_bind_group(0, &pp.fx_state.r_bg, &[]);
        r_pass.set_bind_group(1, &pp.io_ctx.bg, &[]);
        r_pass.draw(0..3, 0..1);
        gfx_state.end_scope(&mut r_pass);

        let profiler = &mut gfx_state.profiler;
        profiler.begin_scope("Render GUI", &mut r_pass, &gfx_state.device);
        gfx_state.renderer.render(
            &mut r_pass,
            &clipped_primitives,
            &gfx_state.screen_descriptor,
        );
        profiler.end_scope(&mut r_pass).unwrap();
    }

    pub fn new(gfx_state: &GfxState, app_settings: &impl AppSettings) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let fx_state = FxState::new(gfx_state);

        let io_uniform = FxIOUniform::zero(&fx_state);
        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "IO");

        let r_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[&fx_state.r_bg_layout, &io_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Finalize pipeline"),
            layout: Some(&r_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &finalize_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
            }),
            multiview: None,
        });

        let effects = app_settings.add_post_fx(&FxOptions {
            fx_state: &fx_state,
            gfx_state,
        });

        Self {
            fx_state,
            effects,

            render_pipeline,

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
        let options = FxOptions {
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

    tex_views: Vec<wgpu::TextureView>,
}

const WORK_GROUP_SIZE: f32 = 16.;

impl FxState {
    pub fn count_in(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let res = (self.tex_size / io_uniform.in_downscale as f32 / WORK_GROUP_SIZE).ceil();

        (res.x as u32, res.y as u32)
    }

    pub fn count_out(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let res = (self.tex_size / io_uniform.out_downscale as f32 / WORK_GROUP_SIZE).ceil();

        (res.x as u32, res.y as u32)
    }

    fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
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
            // Depth
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
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        let r_layout_entries = [
            // Fx read
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
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
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
            tex_views,
        }
    }
}
