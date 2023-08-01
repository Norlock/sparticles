use super::AppState;
use crate::traits::CreateGui;
use egui::{Color32, Context, RichText};

impl CreateGui for Context {
    fn create_gui(&self, app_state: &AppState) {
        let clock = &app_state.clock;

        let fps_text = RichText::new(&clock.fps_text).color(Color32::WHITE);
        let cpu_time_text = RichText::new(&clock.cpu_time_text).color(Color32::WHITE);

        egui::Window::new("Emitter settings").show(self, |ui| {
            ui.label(fps_text);
            ui.label(cpu_time_text);
        });
    }
}
