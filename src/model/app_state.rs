use crate::InitApp;

use super::{Camera, Clock, GfxState, GuiState, SpawnState};
use egui_wgpu::wgpu;
use egui_winit::winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub clock: Clock,
    pub light_spawner: SpawnState,
    pub spawners: Vec<SpawnState>,
    pub gui: GuiState,
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
    }

    pub fn process_events(&mut self, input: KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn compute<'a>(&'a self, compute_pass: &mut wgpu::ComputePass<'a>) {
        self.light_spawner.compute(&self.clock, compute_pass);

        for spawner in self.spawners.iter() {
            spawner.compute(&self.clock, compute_pass);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.light_spawner
            .render_light(&self.clock, &self.camera, render_pass);

        for spawner in self.spawners.iter() {
            spawner.render(&self.clock, &self.camera, &self.light_spawner, render_pass);
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

        AppState {
            clock,
            camera,
            spawners,
            light_spawner,
            gui,
        }
    }
}
