use egui::Context;

pub struct GuiState;

impl GuiState {
    pub fn create_gui(ctx: &Context) {
        egui::Window::new("Emitter settings").show(ctx, |ui| {
            ui.label("Hello World!");
        });
    }
}
