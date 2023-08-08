use crate::{texture::DepthTexture, traits::Animation, InitialiseApp};

use super::{Camera, Clock, GfxState, GuiState, ParticleState};
use egui_wgpu::wgpu;
use egui_winit::winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub clock: Clock,
    pub depth_texture: DepthTexture,
    pub particle: ParticleState,
}

impl AppState {
    pub fn update(&mut self, gfx_state: &GfxState, gui_state: &GuiState) {
        self.clock.update();
        self.camera.update(gfx_state, &self.clock);
        self.particle.update(gfx_state, &self.clock);

        self.handle_gui(gfx_state, gui_state);
    }

    pub fn handle_gui(&mut self, gfx_state: &GfxState, gui_state: &GuiState) {
        if !gui_state.show {
            return;
        }

        self.camera.handle_gui(gui_state);
        self.particle.handle_gui(gui_state, gfx_state, &self.camera);
    }

    pub fn window_resize(&mut self, gfx_state: &GfxState) {
        self.camera.window_resize(&gfx_state);
        self.depth_texture = gfx_state.create_depth_texture();
    }

    pub fn process_events(&mut self, input: KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn compute<'a>(&'a self, compute_pass: &mut wgpu::ComputePass<'a>) {
        self.particle.compute(&self.clock, compute_pass);
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.particle.render(&self.clock, &self.camera, render_pass)
    }
}

impl GfxState {
    pub fn create_app_state(&self, init_app: InitialiseApp) -> AppState {
        let clock = Clock::new();
        let camera = Camera::new(&self);
        let mut particle = self.create_particle_state(init_app.emitter, &camera);

        let animations: Vec<Box<dyn Animation>> = init_app
            .particle_animations
            .into_iter()
            .map(|anim| anim.create_animation(&self, &particle))
            .collect();

        particle.set_animations(animations);

        let depth_texture = self.create_depth_texture();

        AppState {
            clock,
            camera,
            particle,
            depth_texture,
        }
    }
}
