use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;
use std::num::NonZeroU64;

use crate::{
    model::{Camera, GfxState, SparState},
    shaders::{ShaderOptions, SDR_TONEMAPPING},
    traits::BufferContent,
};

#[derive(ShaderType)]
pub struct TerrainUniform {
    pub noise: f32,
    pub elapsed: f32,
}

pub struct TerrainGenerator {
    pub render_pipeline: wgpu::RenderPipeline,
    pub buf: wgpu::Buffer,
    pub bg: wgpu::BindGroup,
    pub uniform: TerrainUniform,
}

const SDR_NOISE: &str = "noise.wgsl";
const SDR_TERRAIN: &str = "terrain/terrain.wgsl";

impl TerrainGenerator {
    pub fn render(state: &SparState, encoder: &mut wgpu::CommandEncoder) {
        let tg = &state.terrain_generator;
        let pp = &state.post_process;
        let camera = &state.camera;

        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pp.frame_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: pp.depth_view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        r_pass.set_pipeline(&tg.render_pipeline);
        r_pass.set_bind_group(0, camera.bg(), &[]);
        r_pass.set_bind_group(1, &tg.bg, &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub async fn new(gfx: &GfxState, camera: &Camera) -> Self {
        let device = &gfx.device;

        let terrain_uniform = TerrainUniform {
            noise: 0.5,
            elapsed: 1.0,
        };

        let buffer_content = terrain_uniform.buffer_content();

        let mut layout_entries = Vec::new();
        let mut entries = Vec::new();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: Some("Terrain generator"),
            contents: &buffer_content,
        });

        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(buffer_content.len() as u64),
            },
            count: None,
        });

        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Terrain bind group layout"),
            entries: &layout_entries,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Terrain bind group"),
            layout: &bind_group_layout,
            entries: &entries,
        });

        let shader = gfx.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &[SDR_TONEMAPPING, SDR_NOISE, SDR_TERRAIN],
            label: "Terrain SDR",
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Generator Pipeline Layout"),
            bind_group_layouts: &[&camera.bg_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: GfxState::TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: GfxState::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
            buf: buffer,
            bg: bind_group,
            uniform: terrain_uniform,
        }
    }
}
