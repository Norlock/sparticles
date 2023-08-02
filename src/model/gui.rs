use super::AppState;
use crate::InitialiseApp;
use egui::{Color32, Context, RichText};

pub struct GuiState {
    show: bool,
    fps_text: String,
    cpu_time_text: String,
}

impl GuiState {
    pub fn new(init_app: &InitialiseApp) -> Self {
        Self {
            show: init_app.show_gui,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
        }
    }

    pub fn create_gui(&mut self, app_state: &mut AppState, ctx: &Context) {
        if !self.show {
            return;
        }

        let clock = &app_state.clock;

        if clock.frame() % 20 == 0 {
            self.cpu_time_text = clock.cpu_time_text();
            self.fps_text = clock.fps_text();
        }

        let fps_text = RichText::new(&self.fps_text).color(Color32::WHITE);
        let cpu_time_text = RichText::new(&self.cpu_time_text).color(Color32::WHITE);

        egui::Window::new("Emitter settings").show(ctx, |ui| {
            ui.label(fps_text);
            ui.label(cpu_time_text);

            if ui.button("Reset camera").clicked() {
                app_state.camera.reset();
            }
        });
    }
}
