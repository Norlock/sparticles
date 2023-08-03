use super::AppState;
use crate::InitialiseApp;
use egui::{Color32, Context, RichText, Ui};
use glam::Vec3;

pub struct GuiState {
    show: bool,
    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    box_yaw: f32,
    box_pitch: f32,
    box_roll: f32,
    diff_width: f32,
    diff_depth: f32,
}

impl GuiState {
    pub fn new(init_app: &InitialiseApp) -> Self {
        Self {
            show: init_app.show_gui,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            box_yaw: 0.,
            box_pitch: 0.,
            box_roll: 0.,
            diff_width: 0.,
            diff_depth: 0.,
        }
    }

    fn update_labels(&mut self, app_state: &AppState) {
        let clock = &app_state.clock;

        if clock.frame() % 20 != 0 {
            return;
        }

        self.cpu_time_text = clock.cpu_time_text();
        self.fps_text = clock.fps_text();
        self.elapsed_text = clock.elapsed_text();
    }

    pub fn create_gui(&mut self, app_state: &mut AppState, ctx: &Context) {
        if !self.show {
            return;
        }

        self.update_labels(app_state);

        let emitter = &mut app_state.compute.emitter;

        egui::Window::new("Emitter settings").show(ctx, |ui| {
            create_label(ui, &self.fps_text);
            create_label(ui, &self.cpu_time_text);
            create_label(ui, &self.elapsed_text);

            if ui.button("Reset camera").clicked() {
                app_state.camera.reset();
            }
            ui.add_space(5.0);

            ui.add(egui::Slider::new(&mut self.box_yaw, 0.0..=360.0).text("Box yaw"));
            ui.add(egui::Slider::new(&mut self.box_pitch, 0.0..=360.0).text("Box pitch"));
            ui.add(egui::Slider::new(&mut self.box_roll, 0.0..=360.0).text("Box roll"));
            ui.add(egui::Slider::new(&mut self.diff_width, 0.0..=360.0).text("Diffusion width"));
            ui.add(egui::Slider::new(&mut self.diff_depth, 0.0..=360.0).text("Diffusion depth"));

            ui.add_space(5.0);
            create_label(ui, "Box dimensions (w, h, d)");

            ui.horizontal(|ui| {
                create_drag_value(ui, &mut emitter.box_dimensions.x);
                create_drag_value(ui, &mut emitter.box_dimensions.y);
                create_drag_value(ui, &mut emitter.box_dimensions.z);
            });
        });

        emitter.box_rotation = Vec3::new(
            self.box_yaw.to_radians(),
            self.box_pitch.to_radians(),
            self.box_roll.to_radians(),
        );

        emitter.diffusion_width_rad = self.diff_width.to_radians();
        emitter.diffusion_depth_rad = self.diff_depth.to_radians();
    }
}

fn create_label(ui: &mut Ui, str: &str) {
    ui.label(RichText::new(str).color(Color32::WHITE));
    ui.add_space(5.0);
}

fn create_drag_value(ui: &mut Ui, val: &mut f32) {
    ui.add(
        egui::DragValue::new(val)
            .clamp_range(0f64..=f64::MAX)
            .speed(0.1),
    );
}
