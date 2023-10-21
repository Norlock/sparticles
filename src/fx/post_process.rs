use crate::init::AppSettings;
use crate::model::{GfxState, State};
use crate::traits::*;
use crate::util::{CommonBuffer, DynamicExport, ExportType, ListAction, Persistence};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};
use std::num::{NonZeroU32, NonZeroU64};

pub struct PostProcessState {
    pub effects: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,

    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
}

pub struct PingPongState {
    fx_idx: usize,
    blend_idx: usize,
}

impl PingPongState {
    fn new() -> Self {
        Self {
            fx_idx: 0,
            blend_idx: 0,
        }
    }

    fn idx(&self) -> usize {
        self.blend_idx * 2 + self.fx_idx
    }

    fn swap_fx(&mut self) {
        self.fx_idx = (self.fx_idx + 1) % 2;
    }

    fn swap_blend(&mut self) {
        self.blend_idx = (self.blend_idx + 1) % 2;
    }

    pub fn swap(&mut self, meta_uniform: &FxMetaUniform) {
        if meta_uniform.out_idx == 0 {
            self.swap_blend();
        } else {
            self.swap_fx();
        }
    }
}

#[derive(ShaderType, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct FxMetaUniform {
    in_idx: u32,
    out_idx: u32,
}

impl FxMetaUniform {
    pub fn new(in_idx: u32, out_idx: u32) -> Self {
        Self { in_idx, out_idx }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

pub struct CreateFxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
}

pub struct BufferInfo {
    pub buffer: wgpu::Buffer,
    pub binding_size: Option<NonZeroU64>,
}

impl FxMetaUniform {
    pub fn create_content(&self) -> Vec<u8> {
        CommonBuffer::uniform_content(self)
    }
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.fx_state = FxState::new(gfx_state);
    }

    pub fn update(state: &mut State) {
        let effects = &mut state.post_process.effects;

        for fx in effects.iter_mut() {
            fx.update(&state.gfx_state);
        }

        // TODO make function in item action to handle all cases like this
        let mut i = 0;

        while i < effects.len() {
            if effects[i].selected_action() == &mut ListAction::Delete {
                effects.remove(i);
                continue;
            } else if 0 < i && effects[i].selected_action() == &mut ListAction::MoveUp {
                effects[i].reset_action();
                effects.swap(i, i - 1);
            } else if 0 < i && effects[i - 1].selected_action() == &mut ListAction::MoveDown {
                effects[i - 1].reset_action();
                effects.swap(i, i - 1);
            }

            i += 1;
        }
    }

    pub fn compute(state: &mut State, encoder: &mut wgpu::CommandEncoder) -> PingPongState {
        let pp = &mut state.post_process;
        let fx_state = &mut pp.fx_state;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
            timestamp_writes: None,
        });

        let mut ping_pong = PingPongState::new();

        c_pass.set_pipeline(&pp.initialize_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(&mut ping_pong), &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        ping_pong.swap_blend();

        for fx in pp.effects.iter().filter(|fx| fx.enabled()) {
            fx.compute(&mut ping_pong, &fx_state, &mut c_pass);
        }

        ping_pong
    }

    pub fn render(
        state: &mut State,
        output_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        mut ping_pong: PingPongState,
    ) {
        let clipped_primitives = GfxState::draw_gui(state, encoder);

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
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let pp = &mut state.post_process;
        r_pass.set_pipeline(&pp.finalize_pipeline);
        r_pass.set_bind_group(0, pp.fx_state.bind_group(&mut ping_pong), &[]);
        r_pass.draw(0..3, 0..1);

        state.gfx_state.renderer.render(
            &mut r_pass,
            &clipped_primitives,
            &state.gfx_state.screen_descriptor,
        );
    }

    pub fn new(gfx_state: &GfxState, app_settings: &impl AppSettings) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let initialize_shader = device.create_shader("fx/initialize.wgsl", "Init post fx");
        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let fx_state = FxState::new(gfx_state);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx layout"),
            bind_group_layouts: &[&fx_state.bind_group_layout],
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

        let effects = app_settings.add_post_fx(&CreateFxOptions {
            fx_state: &fx_state,
            gfx_state,
        });

        Self {
            fx_state,
            effects,
            initialize_pipeline,
            finalize_pipeline,
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
}

pub type Dimensions = [u32; 2];

impl FxState {
    pub fn bind_group(&self, ping_pong: &mut PingPongState) -> &wgpu::BindGroup {
        &self.bind_groups[ping_pong.idx()]
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
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
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
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post process layout"),
            entries: &layout_entries,
        });

        let mut ping_views = Vec::new();
        let mut pong_views = Vec::new();

        for _ in 0..(array_count - 1) {
            ping_views.push(gfx_state.create_fx_view());
            pong_views.push(gfx_state.create_fx_view());
        }

        let mut bind_groups = Vec::new();

        let blend_view_1 = gfx_state.create_fx_view();
        let blend_view_2 = gfx_state.create_fx_view();

        // Make 4 bind groups
        for i in 0..4 {
            let blend_view_ping;
            let blend_view_pong;

            if i < 2 {
                blend_view_ping = &blend_view_1;
                blend_view_pong = &blend_view_2;
            } else {
                blend_view_ping = &blend_view_2;
                blend_view_pong = &blend_view_1;
            };

            let mut ping_refs = vec![blend_view_ping];
            let mut pong_refs = vec![blend_view_pong];

            for ping_ref in ping_views.iter() {
                ping_refs.push(ping_ref);
            }

            for pong_ref in pong_views.iter() {
                pong_refs.push(pong_ref);
            }

            let all_refs = [ping_refs, pong_refs];

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
                ],
            }));
        }

        let count_x = (config.width as f32 / WORK_GROUP_SIZE[0]).ceil() as u32;
        let count_y = (config.height as f32 / WORK_GROUP_SIZE[1]).ceil() as u32;

        Self {
            bind_groups,
            bind_group_layout,
            count_x,
            count_y,
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
