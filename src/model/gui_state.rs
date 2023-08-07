use super::{emitter::SpawnOptions, AppState, GfxState};
use egui::{Color32, Context, RichText, Slider, Ui};
use glam::Vec3;

pub struct GuiState {
    show: bool,
    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,
    box_yaw_deg: f32,
    box_pitch_deg: f32,
    box_roll_deg: f32,
    diff_width_deg: f32,
    diff_depth_deg: f32,
    spawn_count: u32,
    spawn_delay_sec: f32,
    particle_lifetime_sec: f32,
}

impl GuiState {
    pub fn new(show_gui: bool, app_state: &AppState) -> Self {
        let emitter = &app_state.compute.emitter;
        let spawn_options = emitter.get_spawn_options();

        Self {
            show: show_gui,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            box_yaw_deg: emitter.box_rotation.x.to_degrees(),
            box_pitch_deg: emitter.box_rotation.y.to_degrees(),
            box_roll_deg: emitter.box_rotation.z.to_degrees(),
            diff_width_deg: emitter.diffusion_width_rad.to_degrees(),
            diff_depth_deg: emitter.diffusion_depth_rad.to_degrees(),
            spawn_count: spawn_options.spawn_count,
            spawn_delay_sec: spawn_options.spawn_delay_sec,
            particle_lifetime_sec: spawn_options.particle_lifetime_sec,
        }
    }

    fn update_labels(&mut self, app_state: &AppState) {
        let clock = &app_state.clock;
        let compute = &app_state.compute;

        if clock.frame() % 20 != 0 {
            return;
        }

        self.cpu_time_text = clock.cpu_time_text();
        self.fps_text = clock.fps_text();
        self.elapsed_text = clock.elapsed_text();
        self.particle_count_text = compute.particle_count_text();
    }

    pub fn update(&mut self, app_state: &mut AppState, gfx_state: &GfxState, ctx: &Context) {
        if !self.show {
            return;
        }

        self.update_labels(app_state);

        egui::Window::new("Emitter settings").show(&ctx, |ui| {
            let emitter = &mut app_state.compute.emitter;

            create_label(ui, &self.fps_text);
            create_label(ui, &self.cpu_time_text);
            create_label(ui, &self.elapsed_text);
            create_label(ui, &self.particle_count_text);

            if ui.button("Reset camera").clicked() {
                app_state.camera.reset();
            }

            ui.add_space(5.0);

            ui.add(Slider::new(&mut self.box_yaw_deg, 0.0..=360.0).text("Box yaw"));
            ui.add(Slider::new(&mut self.box_pitch_deg, 0.0..=360.0).text("Box pitch"));
            ui.add(Slider::new(&mut self.box_roll_deg, 0.0..=360.0).text("Box roll"));
            ui.add(Slider::new(&mut self.diff_width_deg, 0.0..=360.0).text("Diffusion width"));
            ui.add(Slider::new(&mut self.diff_depth_deg, 0.0..=360.0).text("Diffusion depth"));

            ui.add_space(5.0);
            create_label(ui, "Box dimensions (w, h, d)");

            ui.horizontal(|ui| {
                create_drag_value(ui, &mut emitter.box_dimensions.x);
                create_drag_value(ui, &mut emitter.box_dimensions.y);
                create_drag_value(ui, &mut emitter.box_dimensions.z);
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

            if ui.button("Update spawn settings").clicked() {
                app_state.new_compute_state(
                    gfx_state,
                    SpawnOptions {
                        spawn_count: self.spawn_count,
                        spawn_delay_sec: self.spawn_delay_sec,
                        particle_lifetime_sec: self.particle_lifetime_sec,
                    },
                );
            }

            ui.add_space(5.0);
        });

        let emitter = &mut app_state.compute.emitter;
        emitter.box_rotation = Vec3::new(
            self.box_yaw_deg.to_radians(),
            self.box_pitch_deg.to_radians(),
            self.box_roll_deg.to_radians(),
        );

        emitter.diffusion_width_rad = self.diff_width_deg.to_radians();
        emitter.diffusion_depth_rad = self.diff_depth_deg.to_radians();

        return;
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
