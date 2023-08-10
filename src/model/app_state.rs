use crate::{traits::Animation, InitialiseApp};

use super::{Camera, Clock, GfxState, GuiState, SpawnState};
use egui_wgpu::wgpu;
use egui_winit::winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub clock: Clock,
    pub spawners: Vec<SpawnState>,
}

impl AppState {
    pub fn update(&mut self, gfx_state: &GfxState, gui_state: &GuiState) {
        self.clock.update();
        self.camera.update(gfx_state, &self.clock);

        for spawner in self.spawners.iter_mut() {
            spawner.update(gfx_state, &self.clock);
        }

        self.handle_gui(gfx_state, gui_state);
    }

    pub fn handle_gui(&mut self, gfx_state: &GfxState, gui_state: &GuiState) {
        if !gui_state.show {
            return;
        }

        self.camera.handle_gui(gui_state);

        let spawner = self
            .spawners
            .iter_mut()
            .find(|s| s.id == gui_state.selected_spawner_id);

        if let Some(spawner) = spawner {
            spawner.handle_gui(gfx_state, &self.camera);
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
        for spawner in self.spawners.iter() {
            spawner.compute(&self.clock, compute_pass);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for spawner in self.spawners.iter() {
            spawner.render(&self.clock, &self.camera, render_pass);
        }
    }
}

impl GfxState {
    pub fn create_app_state(&self, init_app: InitialiseApp) -> AppState {
        let clock = Clock::new();
        let camera = Camera::new(&self);

        let mut spawners: Vec<SpawnState> = Vec::new();

        for item in init_app.spawners {
            let mut spawner = self.create_spawner(item.emitter, &camera, item.id);
            assert!(!spawner.id.is_empty(), "Id can not be empty");

            let animations: Vec<Box<dyn Animation>> = item
                .particle_animations
                .into_iter()
                .map(|anim| anim.into_animation(&self, &spawner))
                .collect();

            spawner.set_animations(animations);

            let is_unique = spawners.iter().find(|s| spawner.id == s.id).is_none();
            assert!(is_unique, "Spawners require an unique ID");

            spawners.push(spawner);
        }

        AppState {
            clock,
            camera,
            spawners,
        }
    }
}
