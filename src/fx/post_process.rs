use crate::init::AppSettings;
use crate::model::{GfxState, State};
use crate::traits::*;
use crate::util::CommonBuffer;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::util::DeviceExt;
use encase::ShaderType;
use serde::{Deserialize, Serialize};
use std::num::{NonZeroU32, NonZeroU64};

pub struct PostProcessState {
    pub post_fx: Vec<Box<dyn PostFx>>,
    pub fx_state: FxState,

    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
}

#[derive(ShaderType, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct FxMetaUniform {
    pub in_idx: i32,  // -1 == fx_blend
    pub out_idx: i32, // -1 == fx_blend
}

impl FxMetaUniform {
    pub fn new(in_idx: i32, out_idx: i32) -> Self {
        Self { in_idx, out_idx }
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

pub struct MetaUniformCompute {
    pub buffer: wgpu::Buffer,
    pub uniform: FxMetaUniform,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl FxMetaUniform {
    pub fn create_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let contents = CommonBuffer::uniform_content(&self);

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta uniform"),
            contents: &contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn into_compute(self, device: &wgpu::Device) -> MetaUniformCompute {
        let contents = CommonBuffer::uniform_content(&self);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta uniform"),
            contents: &contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blur uniform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(contents.len() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        MetaUniformCompute {
            buffer,
            bind_group,
            bind_group_layout,
            uniform: self,
        }
    }
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.fx_state = FxState::new(gfx_state);
    }

    // TODO return idx of bindgroup
    pub fn compute(state: &mut State, encoder: &mut wgpu::CommandEncoder) -> usize {
        let pp = &mut state.post_process;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
            timestamp_writes: None,
        });

        c_pass.set_pipeline(&pp.initialize_pipeline);
        c_pass.set_bind_group(0, pp.fx_state.bind_group(0), &[]);
        c_pass.dispatch_workgroups(pp.fx_state.count_x, pp.fx_state.count_y, 1);

        let mut ping_pong_idx = 0;

        for fx in pp.post_fx.iter().filter(|fx| fx.enabled()) {
            fx.compute(&mut ping_pong_idx, &pp.fx_state, &mut c_pass);
        }

        ping_pong_idx
    }

    pub fn render(
        state: &mut State,
        output_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        ping_pong_idx: usize,
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
        r_pass.set_bind_group(0, pp.fx_state.bind_group(ping_pong_idx), &[]);
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

        let post_fx = app_settings.add_post_fx(&CreateFxOptions {
            fx_state: &fx_state,
            gfx_state,
        });

        Self {
            fx_state,
            post_fx,
            initialize_pipeline,
            finalize_pipeline,
        }
    }

    pub fn import_fx(&mut self, gfx_state: &GfxState) {
        // TODO
    }

    pub fn export(pp: &PostProcessState) {
        //Persistence::write_to_file(to_export, ExportType::PostFx);
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
    pub fn bind_group(&self, idx: usize) -> &wgpu::BindGroup {
        &self.bind_groups[idx % 2]
    }

    fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;
        let frame_view = gfx_state.create_frame_view();
        let depth_view = gfx_state.create_depth_view();

        let array_count = 16;

        let mut layout_entries = Vec::new();

        // Fx write
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                view_dimension: wgpu::TextureViewDimension::D2,
                format: PostProcessState::TEXTURE_FORMAT,
                access: wgpu::StorageTextureAccess::WriteOnly,
            },
            count: NonZeroU32::new(array_count),
        });

        // Fx Read
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                multisampled: false,
            },
            count: NonZeroU32::new(array_count),
        });

        // Fx Blend READ_WRITE
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::StorageTexture {
                view_dimension: wgpu::TextureViewDimension::D2,
                format: PostProcessState::TEXTURE_FORMAT,
                access: wgpu::StorageTextureAccess::ReadWrite,
            },
            count: None,
        });

        // Frame
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                multisampled: false,
            },
            count: None,
        });

        // Depth
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                multisampled: false,
            },
            count: None,
        });

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
        let blend_view = gfx_state.create_fx_view();

        let ping_refs: Vec<&wgpu::TextureView> = ping_views.iter().map(|v| v).collect();
        let pong_refs: Vec<&wgpu::TextureView> = pong_views.iter().map(|v| v).collect();
        let all_refs = [&ping_refs, &pong_refs];

        for i in 0..2 {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fx bindgroup"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(all_refs[i % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureViewArray(all_refs[(i + 1) % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&blend_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&depth_view),
                    },
                ],
            }));
        }

        // Render layout

        // Render bind_group

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
