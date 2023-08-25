use crate::{texture::DepthTexture, InitApp};

use super::{Bloom, Camera, Clock, GfxState, GuiState, SpawnState};
use egui_wgpu::{wgpu, Renderer};
use egui_winit::winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub clock: Clock,
    pub light_spawner: SpawnState,
    pub spawners: Vec<SpawnState>,
    pub gui: GuiState,
    pub post_process: Bloom,
}

impl AppState {
    pub fn update(&mut self, gfx_state: &GfxState) {
        self.clock.update();
        self.camera.update(gfx_state, &self.clock);
        self.handle_gui(gfx_state);

        self.light_spawner.update(gfx_state, &self.clock);

        for spawner in self.spawners.iter_mut() {
            spawner.update(gfx_state, &self.clock);
        }
    }

    pub fn handle_gui(&mut self, gfx_state: &GfxState) {
        if !self.gui.show {
            return;
        }

        self.camera.handle_gui(&self.gui);

        if self.light_spawner.id == self.gui.selected_id {
            self.light_spawner.handle_gui(gfx_state, None, &self.camera);
        } else {
            let light_layout = &self.light_spawner.bind_group_layout;

            let selected = self
                .spawners
                .iter_mut()
                .find(|s| s.id == self.gui.selected_id);

            if let Some(spawner) = selected {
                spawner.handle_gui(gfx_state, Some(light_layout), &self.camera);
            }
        }
    }

    pub fn particle_count_text(&self) -> String {
        let particle_count: u64 = self.spawners.iter().map(|s| s.particle_count()).sum();

        format!("Particle count: {}", particle_count)
    }

    pub fn window_resize(&mut self, gfx_state: &GfxState) {
        self.camera.window_resize(&gfx_state);
        self.post_process.resize(&gfx_state);
    }

    pub fn process_events(&mut self, input: KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn compute<'a>(&'a self, encoder: &mut wgpu::CommandEncoder) {
        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pipeline"),
        });

        self.light_spawner.compute(&self.clock, &mut c_pass);

        for spawner in self.spawners.iter() {
            spawner.compute(&self.clock, &mut c_pass);
        }
    }

    pub fn compute_fx(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
        });

        self.post_process.compute_fx(&mut c_pass);
        drop(c_pass);
    }

    pub fn render_fx<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        self.post_process.render_fx(r_pass);
    }

    pub fn render<'a>(&'a self, encoder: &mut wgpu::CommandEncoder, depth_texture: &DepthTexture) {
        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.post_process.res.frame_tex_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.light_spawner
            .render_light(&self.clock, &self.camera, &mut r_pass);

        for spawner in self.spawners.iter() {
            spawner.render(&self.clock, &self.camera, &self.light_spawner, &mut r_pass);
        }
    }
}

impl GfxState {
    pub fn create_app_state(&self, mut init_app: InitApp) -> AppState {
        let show_gui = init_app.show_gui;
        let clock = Clock::new();
        let camera = Camera::new(&self);
        let light_spawner = init_app.create_light_spawner(&self, &camera);
        let spawners = init_app.create_spawners(&self, &light_spawner.bind_group_layout, &camera);

        let gui = GuiState::new(&spawners, show_gui);
        let post_process = self.create_post_process();

        AppState {
            clock,
            camera,
            spawners,
            light_spawner,
            gui,
            post_process,
        }
    }
}
