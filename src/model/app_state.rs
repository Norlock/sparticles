use crate::texture::DiffuseTexture;

use super::{gfx_state, Camera, Clock};
use egui_wgpu_backend::wgpu;

pub struct AppState {
    pub camera: Camera,
    pub diffuse_texture: DiffuseTexture,
    pub clock: Clock,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl AppState {
    pub fn new(gfx_state: &gfx_state::GfxState) -> Self {
        let device = &gfx_state.device;
        let surface_config = &gfx_state.surface_config;

        let clock = Clock::new();
        let camera = Camera::new(gfx_state);
        let diffuse_texture = DiffuseTexture::new(&gfx_state);

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/particle.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Empty Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &camera.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            clock,
            camera,
            render_pipeline,
            diffuse_texture,
        }
    }

    pub fn update(&mut self, gfx_state: &gfx_state::GfxState) {
        self.clock.update();
        self.camera.update(gfx_state);
    }

    pub fn window_resize(&mut self, gfx_state: &gfx_state::GfxState) {
        self.camera.window_resize(&gfx_state);
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
