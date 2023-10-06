use super::{Camera, Clock, EmitterState, GfxState, GuiState};
use crate::init::{InitEmitters, InitSettings};
use crate::traits::*;
use crate::{fx::PostProcessState, util::Persistence, AppSettings};
use egui_wgpu::wgpu;
use egui_winit::winit::{dpi::PhysicalSize, event::KeyboardInput, window::Window};

pub struct State {
    pub camera: Camera,
    pub clock: Clock,
    pub lights: EmitterState,
    pub emitters: Vec<EmitterState>,
    pub gui: GuiState,
    pub post_process: PostProcessState,
    pub gfx_state: GfxState,
    pub registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registered_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
}

pub enum Messages {
    ResetCamera,
    RemovePostFx,
}

impl State {
    pub fn update(&mut self) {
        self.clock.update();

        Camera::update(self);
        EmitterState::update_spawners(self);
        GuiState::process_gui(self);
    }

    pub fn render(&mut self) {
        GfxState::render(self);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gfx_state.resize(size);
        self.post_process.resize(&self.gfx_state);
        self.camera.resize(&self.gfx_state);
    }

    pub fn process_events(&mut self, input: KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn frame_view(&self) -> &wgpu::TextureView {
        &self.post_process.frame_state.frame_view
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.post_process.frame_state.depth_view
    }

    pub fn new(app_settings: impl AppSettings, window: Window) -> Self {
        let gfx_state = pollster::block_on(GfxState::new(window));

        let clock = Clock::new();
        let camera = Camera::new(&gfx_state);

        let InitEmitters {
            lights,
            emitters,
            registered_em_anims,
            registered_par_anims,
        } = InitSettings::create_emitters(&app_settings, &gfx_state, &camera);

        let mut post_process = PostProcessState::new(&gfx_state);

        // TODO look at import type
        if let Ok(fx_types) = Persistence::import_post_fx() {
            post_process.import_fx(&gfx_state, fx_types);
        } else {
            post_process.add_default_fx(&gfx_state);
        }

        let gui = GuiState::new(app_settings.show_gui());

        Self {
            clock,
            camera,
            emitters,
            lights,
            gui,
            post_process,
            gfx_state,
            registered_par_anims,
            registered_em_anims,
        }
    }
}
