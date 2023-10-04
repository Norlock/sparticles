use super::{Camera, Clock, EmitterState, GfxState, GuiState};
use crate::{fx::PostProcessState, util::Persistence, InitApp};
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
        GuiState::handle_gui(self);
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

    pub fn new(mut init_app: InitApp, window: Window) -> Self {
        let gfx_state = pollster::block_on(GfxState::new(window));
        let show_gui = init_app.show_gui;
        let clock = Clock::new();
        let camera = Camera::new(&gfx_state);
        let light_spawner = init_app.create_light_spawner(&gfx_state, &camera);
        let spawners =
            init_app.create_spawners(&gfx_state, &light_spawner.bind_group_layout, &camera);

        let mut post_process = PostProcessState::new(&gfx_state);

        if let Ok(fx_types) = Persistence::fetch_post_fx() {
            post_process.import_fx(&gfx_state, fx_types);
        } else {
            post_process.add_default_fx(&gfx_state);
        }

        let gui = GuiState::new(&spawners, show_gui);

        Self {
            clock,
            camera,
            emitters: spawners,
            lights: light_spawner,
            gui,
            post_process,
            gfx_state,
        }
    }
}
