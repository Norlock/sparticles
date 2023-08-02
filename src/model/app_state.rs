use crate::{
    texture::DiffuseTexture,
    traits::{Animation, CustomShader},
    InitialiseApp,
};

use super::{gfx_state::GfxState, Camera, Clock, ComputeState};
use egui_wgpu_backend::wgpu;
use winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub diffuse_texture: DiffuseTexture,
    pub clock: Clock,

    render_pipeline: wgpu::RenderPipeline,
    compute: ComputeState,
    animations: Vec<Box<dyn Animation>>,
}

impl AppState {
    pub fn update(&mut self, gfx_state: &GfxState) {
        self.clock.update();
        self.camera.update(gfx_state, &self.clock);
        self.compute.update(gfx_state, &self.clock);

        for anim in self.animations.iter_mut() {
            anim.update(&self.clock, gfx_state);
        }
    }

    pub fn window_resize(&mut self, gfx_state: &GfxState) {
        self.camera.window_resize(&gfx_state);
    }

    pub fn process_events(&mut self, input: KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn compute<'a>(&'a self, compute_pass: &mut wgpu::ComputePass<'a>) {
        self.compute.compute(&self.clock, compute_pass);

        for anim in self.animations.iter() {
            anim.compute(&self.clock, &self.compute, compute_pass);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let nr = self.clock.get_alt_bindgroup_nr();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(2, &self.compute.bind_groups[nr], &[]);
        render_pass.draw(0..4, 0..self.compute.particle_count() as u32);
    }
}

impl GfxState {
    pub fn create_app_state(&self, init_app: InitialiseApp) -> AppState {
        let clock = Clock::new();
        let camera = Camera::new(&self);
        let diffuse_texture = self.create_diffuse_texture();
        let compute = self.create_compute_state();
        let render_pipeline = self.create_render_pipeline(&diffuse_texture, &camera, &compute);

        let animations: Vec<Box<dyn Animation>> = init_app
            .particle_animations
            .into_iter()
            .map(|anim| anim.create_animation(&self, &compute))
            .collect();

        AppState {
            clock,
            camera,
            render_pipeline,
            diffuse_texture,
            compute,
            animations,
        }
    }

    fn create_render_pipeline(
        &self,
        diffuse_texture: &DiffuseTexture,
        camera: &Camera,
        compute_state: &ComputeState,
    ) -> wgpu::RenderPipeline {
        let device = &self.device;
        let surface_config = &self.surface_config;

        let shader_str = include_str!("../shaders/particle.wgsl");
        let shader = device.create_shader(shader_str, "Particle render");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Empty Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &camera.bind_group_layout,
                    &compute_state.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        })
    }
}
