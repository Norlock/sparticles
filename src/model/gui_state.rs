use super::{spawn_state::SpawnGuiState, AppState, SpawnState};
use egui::{Color32, Context, RichText, Slider, Ui, Window};
use egui_winit::egui;

pub struct GuiState {
    pub show: bool,
    pub reset_camera: bool,

    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,

    pub selected_spawner_id: String,
}

impl GuiState {
    fn update_labels(&mut self, app_state: &AppState) {
        let clock = &app_state.clock;

        if clock.frame() % 20 != 0 {
            return;
        }

        self.cpu_time_text = clock.cpu_time_text();
        self.fps_text = clock.fps_text();
        self.elapsed_text = clock.elapsed_text();
        self.particle_count_text = app_state.particle_count_text();
    }

    fn create_spawner_menu(&mut self, ui: &mut Ui, emitter: &mut SpawnGuiState) {
        create_deg_slider(ui, &mut emitter.box_rotation_deg.x, "Box yaw");
        create_deg_slider(ui, &mut emitter.box_rotation_deg.y, "Box pitch");
        create_deg_slider(ui, &mut emitter.box_rotation_deg.z, "Box roll");

        create_deg_slider(ui, &mut emitter.diff_width_deg, "Diffusion width");
        create_deg_slider(ui, &mut emitter.diff_depth_deg, "Diffusion depth");

        ui.add_space(5.0);
        create_label(ui, "Box dimensions (w, h, d)");

        ui.horizontal(|ui| {
            create_drag_value(ui, &mut emitter.box_dimensions.x);
            create_drag_value(ui, &mut emitter.box_dimensions.y);
            create_drag_value(ui, &mut emitter.box_dimensions.z);
        });

        ui.add_space(5.0);
        ui.add(Slider::new(&mut emitter.particle_speed, 0.0..=50.0).text("Particle emit speed"));
        ui.add_space(5.0);
        create_label(ui, "Spawn itemings");

        ui.add(
            egui::Slider::new(&mut emitter.particle_lifetime_sec, 1.0..=40.0)
                .drag_value_speed(0.)
                .step_by(0.1)
                .text("Particle lifetime (sec)"),
        );

        ui.add(
            egui::Slider::new(&mut emitter.spawn_delay_sec, 0.1..=20.0)
                .drag_value_speed(0.)
                .step_by(0.1)
                .text("Spawn delay (sec)"),
        );

        ui.add(egui::Slider::new(&mut emitter.spawn_count, 1..=100).text("Spawn count"));

        ui.add_space(5.0);

        emitter.recreate = ui.button("Update spawn settings").clicked();

        ui.add_space(5.0);

        create_label(ui, "Particle settings");

        ui.add_space(5.0);

        ui.add(
            Slider::new(&mut emitter.particle_size_min, 0.1..=1.0).text("Smallest particle size"),
        );
        ui.add(
            Slider::new(
                &mut emitter.particle_size_max,
                emitter.particle_size_min..=2.0,
            )
            .text("Largest particle size"),
        );
    }

    fn create_gui(&mut self, spawners: &mut Vec<SpawnState>, ctx: &Context) {
        Window::new("Emitter settings").show(&ctx, |ui| {
            create_label(ui, &self.fps_text);
            create_label(ui, &self.cpu_time_text);
            create_label(ui, &self.elapsed_text);
            create_label(ui, &self.particle_count_text);

            self.reset_camera = ui.button("Reset camera").clicked();
            ui.add_space(5.0);

            for spawner in spawners {
                if spawner.id == self.selected_spawner_id {
                    self.create_spawner_menu(ui, &mut spawner.gui);
                }
            }
        });
    }

    pub fn update(&mut self, app_state: &mut AppState, ctx: &Context) {
        if self.show {
            self.update_labels(app_state);
            self.create_gui(&mut app_state.spawners, ctx);
        }
    }
}

impl AppState {
    pub fn create_gui_state(&self, show_gui: bool) -> GuiState {
        let mut selected_spawner_id = "".to_string();
        let mut spawn_delay_sec = 0.;
        let mut spawn_count = 0;
        let mut particle_lifetime_sec = 0.;

        GuiState {
            show: show_gui,
            reset_camera: false,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_spawner_id,
        }
    }
}

fn create_deg_slider(ui: &mut Ui, val: &mut f32, str: &str) {
    ui.add(Slider::new(val, 0.0..=360.).text(str));
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
