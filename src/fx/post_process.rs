use crate::init::AppSettings;
use crate::model::{GfxState, State};
use crate::traits::*;
use crate::util::{
    CommonBuffer, DynamicExport, ExportType, ListAction, Persistence, UniformContext,
};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

pub struct PostProcessState {
    pub effects: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,

    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,

    pub io_uniform: FxIOUniform,
    pub io_buf: wgpu::Buffer,
    pub io_bg: wgpu::BindGroup,
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
    pub in_downscale: u32,
    pub out_idx: u32,
    pub out_downscale: u32,
}

impl FxIOUniform {
    pub fn asymetric_unscaled(in_idx: u32, out_idx: u32) -> Self {
        Self {
            in_idx,
            in_downscale: 1,
            out_idx,
            out_downscale: 1,
        }
    }

    pub fn asymetric_downscaled(in_idx: u32, out_idx: u32, downscale: u32) -> Self {
        assert!(1 <= downscale, "Downscale needs to be 1 or higher");

        Self {
            in_idx,
            in_downscale: downscale,
            out_idx,
            out_downscale: downscale,
        }
    }

    pub fn symetric_unscaled(in_out_idx: u32) -> Self {
        Self {
            in_idx: in_out_idx,
            in_downscale: 1,
            out_idx: in_out_idx,
            out_downscale: 1,
        }
    }

    pub fn symetric_downscaled(in_out_idx: u32, downscale: u32) -> Self {
        assert!(1 <= downscale, "Downscale needs to be 1 or higher");

        Self {
            in_idx: in_out_idx,
            in_downscale: downscale,
            out_idx: in_out_idx,
            out_downscale: downscale,
        }
    }

    pub fn zero() -> Self {
        Self::asymetric_unscaled(0, 0)
    }

    pub fn create_content(&self) -> Vec<u8> {
        CommonBuffer::uniform_content(self)
    }
}

pub struct CreateFxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
}

pub const WORK_GROUP_SIZE: [u32; 2] = [8, 8];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        //self.fx_state = FxState::new(gfx_state);
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
        c_pass.set_bind_group(0, fx_state.bind_group(&mut ping_pong), &[]);
        c_pass.set_bind_group(1, &pp.io_bg, &[]);
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

        if gui.preview_enabled {
            r_pass.set_bind_group(0, &pp.fx_state.bind_groups[gui.selected_bind_group], &[]);
        } else {
            r_pass.set_bind_group(0, &pp.fx_state.bind_group(&ping_pong), &[]);
        }

        r_pass.set_bind_group(1, &pp.io_bg, &[]);
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

        let io_uniform = FxIOUniform::zero();
        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "IO");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[&fx_state.bind_group_layout, &io_ctx.bg_layout],
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
            io_buf: io_ctx.buf,
            io_bg: io_ctx.bg,
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
    bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub count_x: u32,
    pub count_y: u32,
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
    pub aspect: f32,
}

pub type Dimensions = [u32; 2];

impl FxState {
    pub fn bind_group(&self, ping_pong: &PingPongState) -> &wgpu::BindGroup {
        &self.bind_groups[ping_pong.idx()]
    }

    pub fn count_out(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let count_x = self.count_x / io_uniform.out_downscale;
        let count_y = self.count_y / io_uniform.out_downscale;

        (self.count_x, self.count_y)
    }

    pub fn count_in(&self, io_uniform: &FxIOUniform) -> (u32, u32) {
        let count_x = self.count_x / io_uniform.in_downscale;
        let count_y = self.count_y / io_uniform.in_downscale;

        (self.count_x, self.count_y)
    }

    fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;
        let frame_view = gfx_state.create_frame_view();
        let depth_view = gfx_state.create_depth_view();

        let array_count = 16;

        let layout_entries = vec![
            // Fx write
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    view_dimension: wgpu::TextureViewDimension::D2,
                    format: PostProcessState::TEXTURE_FORMAT,
                    access: wgpu::StorageTextureAccess::WriteOnly,
                },
                count: NonZeroU32::new(array_count),
            },
            // Fx read
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    multisampled: false,
                },
                count: NonZeroU32::new(array_count),
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
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post process layout"),
            entries: &layout_entries,
        });

        let mut ping_views = Vec::new();
        let mut pong_views = Vec::new();

        for _ in 0..array_count {
            ping_views.push(gfx_state.create_fx_view());
            pong_views.push(gfx_state.create_fx_view());
        }

        let mut bind_groups = Vec::new();

        let ping_refs: Vec<&wgpu::TextureView> = ping_views.iter().collect();
        let pong_refs: Vec<&wgpu::TextureView> = pong_views.iter().collect();

        let all_refs: [&[&wgpu::TextureView]; 2] = [&ping_refs, &pong_refs];

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        for i in 0..2 {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fx bindgroup"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&all_refs[i % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureViewArray(&all_refs[(i + 1) % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));
        }

        let count_x = 2048 / WORK_GROUP_SIZE[0];
        let count_y = 1024 / WORK_GROUP_SIZE[1];

        Self {
            bind_groups,
            bind_group_layout,
            count_x,
            count_y,
            depth_view,
            frame_view,
            aspect: config.width as f32 / config.height as f32,
        }
    }
}
