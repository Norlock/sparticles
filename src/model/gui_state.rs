use super::AppState;
use crate::traits::HandleAngles;
use egui::{Color32, Context, RichText, Slider, Ui, Window};
use egui_winit::egui;
use glam::Vec3;

pub struct GuiState {
    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
    pub show: bool,

    pub box_rotation_deg: Vec3,
    pub box_dimensions: Vec3,
    pub diff_width_deg: f32,
    pub diff_depth_deg: f32,
    pub update_spawn: bool,
    pub reset_camera: bool,

    pub particle_size_min: f32,
    pub particle_size_max: f32,
}

impl GuiState {
    fn update_labels(&mut self, app_state: &AppState) {
        let clock = &app_state.clock;
        let compute = &app_state.particle;

        if clock.frame() % 20 != 0 {
            return;
        }

        self.cpu_time_text = clock.cpu_time_text();
        self.fps_text = clock.fps_text();
        self.elapsed_text = clock.elapsed_text();
        self.particle_count_text = compute.particle_count_text();
    }

    fn create_gui(&mut self, ctx: &Context) {
        Window::new("Emitter settings").show(&ctx, |ui| {
            create_label(ui, &self.fps_text);
            create_label(ui, &self.cpu_time_text);
            create_label(ui, &self.elapsed_text);
            create_label(ui, &self.particle_count_text);

            self.reset_camera = ui.button("Reset camera").clicked();

            ui.add_space(5.0);

            ui.add(Slider::new(&mut self.box_rotation_deg.x, 0.0..=360.0).text("Box yaw"));
            ui.add(Slider::new(&mut self.box_rotation_deg.y, 0.0..=360.0).text("Box pitch"));
            ui.add(Slider::new(&mut self.box_rotation_deg.z, 0.0..=360.0).text("Box roll"));
            ui.add(Slider::new(&mut self.diff_width_deg, 0.0..=360.0).text("Diffusion width"));
            ui.add(Slider::new(&mut self.diff_depth_deg, 0.0..=360.0).text("Diffusion depth"));

            ui.add_space(5.0);
            create_label(ui, "Box dimensions (w, h, d)");

            ui.horizontal(|ui| {
                create_drag_value(ui, &mut self.box_dimensions.x);
                create_drag_value(ui, &mut self.box_dimensions.y);
                create_drag_value(ui, &mut self.box_dimensions.z);
            });

            ui.add_space(5.0);
            create_label(ui, "Spawn settings");

            ui.add(
                egui::Slider::new(&mut self.particle_lifetime_sec, 1.0..=100.0)
                    .drag_value_speed(0.1)
                    .step_by(0.1)
                    .text("Particle lifetime (sec)"),
            );

            ui.add(
                egui::Slider::new(&mut self.spawn_delay_sec, 0.1..=100.0)
                    .drag_value_speed(0.1)
                    .step_by(0.1)
                    .text("Spawn delay (sec)"),
            );

            ui.add(egui::Slider::new(&mut self.spawn_count, 1..=100).text("Spawn count"));

            ui.add_space(5.0);

            self.update_spawn = ui.button("Update spawn settings").clicked();

            ui.add_space(5.0);

            create_label(ui, "Particle settings");

            ui.add_space(5.0);

            ui.add(
                Slider::new(&mut self.particle_size_min, 0.0..=4.0).text("Smallest particle size"),
            );
            ui.add(
                Slider::new(&mut self.particle_size_max, 0.0..=8.0).text("Largest particle size"),
            );
        });
    }

    pub fn update(&mut self, app_state: &AppState, ctx: &Context) {
        if self.show {
            self.update_labels(app_state);
            self.create_gui(ctx);
        }
    }
}

impl AppState {
    pub fn create_gui_state(&self, show_gui: bool) -> GuiState {
        let emitter = &self.particle.emitter;
        let spawn_options = emitter.get_spawn_options();

        GuiState {
            show: show_gui,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            box_rotation_deg: emitter.box_rotation.to_degrees(),
            box_dimensions: emitter.box_dimensions,
            diff_width_deg: emitter.diffusion_width_rad.to_degrees(),
            diff_depth_deg: emitter.diffusion_depth_rad.to_degrees(),
            spawn_count: spawn_options.spawn_count,
            spawn_delay_sec: spawn_options.spawn_delay_sec,
            particle_lifetime_sec: spawn_options.particle_lifetime_sec,
            particle_size_min: emitter.particle_size_min,
            particle_size_max: emitter.particle_size_max,
            update_spawn: false,
            reset_camera: false,
        }
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
