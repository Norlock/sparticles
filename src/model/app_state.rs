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

        // TODO gui seperate logic for each spawner
        for spawner in self.spawners.iter_mut() {
            spawner.handle_gui(gui_state, gfx_state, &self.camera);
        }
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

        let mut spawners = Vec::new();

        for spawner_init in init_app.spawners {
            let mut spawner = self.create_spawner(spawner_init.emitter, &camera);

            let animations: Vec<Box<dyn Animation>> = spawner_init
                .particle_animations
                .into_iter()
                .map(|anim| anim.into_animation(&self, &spawner))
                .collect();

            spawner.set_animations(animations);

            spawners.push(spawner);
        }

        AppState {
            clock,
            camera,
            spawners,
        }
    }
}
