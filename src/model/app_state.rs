use super::{Camera, Clock, GfxState, GuiState, SpawnState};
use crate::{
    fx::{post_process::ImportOptions, PostProcessState},
    util::Persistence,
    InitApp,
};
use egui_wgpu::wgpu;
use egui_winit::winit::event::KeyboardInput;

pub struct AppState {
    pub camera: Camera,
    pub clock: Clock,
    pub light_spawner: SpawnState,
    pub spawners: Vec<SpawnState>,
    pub gui: GuiState,
    pub post_process: PostProcessState,
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

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.post_process.resize(gfx_state);
        self.camera.window_resize(gfx_state);
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

    pub fn apply_fx(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.post_process.compute(encoder);
    }

    pub fn render_fx<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        self.post_process.render(r_pass);
    }

    fn frame_view(&self) -> &wgpu::TextureView {
        &self.post_process.frame_state.frame_view
    }

    fn depth_view(&self) -> &wgpu::TextureView {
        &self.post_process.frame_state.depth_view
    }

    pub fn render<'a>(&'a self, encoder: &mut wgpu::CommandEncoder) {
        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.frame_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view(),
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
        let mut post_process = PostProcessState::new(&self);

        if let Ok(fx_types) = Persistence::fetch_post_fx() {
            post_process.import_fx(ImportOptions {
                fx_types,
                gfx_state: self,
            });
        } else {
            post_process.add_default_fx(self);
        }

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
