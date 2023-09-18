use super::{spawn_state::SpawnGuiState, AppState, GfxState, SpawnState};
use crate::{
    fx::{
        bloom::BloomExport, post_process::FxView, Bloom, ColorProcessing, ColorProcessingUniform,
        FxPersistenceType, PostProcessState,
    },
    util::{persistence::ExportType, Persistence},
};
use egui::{Color32, Context, RichText, Slider, Ui, Window};
use egui_winit::egui::{self, ComboBox};

pub struct GuiState {
    pub enabled: bool,
    pub reset_camera: bool,
    pub selected_spawner_id: String,

    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,
    selected_tab: Tab,
    selected_post_fx: PostFx,
}

#[derive(PartialEq)]
enum Tab {
    SpawnSettings,
    PostFxSettings,
    AnimationSettings,
}

#[derive(PartialEq, Debug)]
enum PostFx {
    Bloom,
    ColorProcessing,
}

struct GuiContext<'a> {
    spawners: &'a mut Vec<SpawnState>,
    light_spawner: &'a mut SpawnState,
    post_process: &'a mut PostProcessState,
    gfx_state: &'a GfxState,
}

impl GuiState {
    pub fn new(spawners: &Vec<SpawnState>, show_gui: bool) -> Self {
        let spawner = spawners.first();

        let selected_id = spawner.map_or("".to_owned(), |s| s.id.to_owned());

        Self {
            enabled: show_gui,
            reset_camera: false,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_spawner_id: selected_id,
            selected_tab: Tab::SpawnSettings,
            selected_post_fx: PostFx::Bloom,
        }
    }

    fn create_spawner_menu(&mut self, ui: &mut Ui, spawn_gui: &mut SpawnGuiState, id: &str) {
        ui.add_space(5.0);
        create_label(ui, id);

        create_deg_slider(ui, &mut spawn_gui.box_rotation_deg.x, "Box yaw");
        create_deg_slider(ui, &mut spawn_gui.box_rotation_deg.y, "Box pitch");
        create_deg_slider(ui, &mut spawn_gui.box_rotation_deg.z, "Box roll");

        create_deg_slider(ui, &mut spawn_gui.diff_width_deg, "Diffusion width");
        create_deg_slider(ui, &mut spawn_gui.diff_depth_deg, "Diffusion depth");

        ui.add_space(5.0);
        create_label(ui, "Box dimensions (w, h, d)");

        ui.horizontal(|ui| {
            create_drag_value(ui, &mut spawn_gui.box_dimensions.x);
            create_drag_value(ui, &mut spawn_gui.box_dimensions.y);
            create_drag_value(ui, &mut spawn_gui.box_dimensions.z);
        });

        ui.add_space(5.0);
        ui.add(
            Slider::new(&mut spawn_gui.particle_speed_min, 0.0..=50.0)
                .text("Particle emit speed min"),
        );
        ui.add(
            Slider::new(
                &mut spawn_gui.particle_speed_max,
                spawn_gui.particle_speed_min..=50.0,
            )
            .text("Particle emit speed max"),
        );
        ui.add_space(5.0);
        create_label(ui, "Spawn itemings");

        ui.add(
            egui::Slider::new(&mut spawn_gui.particle_lifetime_sec, 1.0..=40.0)
                .drag_value_speed(0.)
                .max_decimals(1)
                .step_by(0.1)
                .text("Particle lifetime (sec)"),
        );

        ui.add(
            egui::Slider::new(&mut spawn_gui.spawn_delay_sec, 0.1..=20.0)
                .drag_value_speed(0.)
                .max_decimals(1)
                .step_by(0.1)
                .text("Spawn delay (sec)"),
        );

        ui.add(egui::Slider::new(&mut spawn_gui.spawn_count, 1..=100).text("Spawn count"));

        ui.add_space(5.0);

        spawn_gui.recreate = ui.button("Update spawn settings").clicked();

        ui.add_space(5.0);

        create_label(ui, "Particle settings");

        ui.add_space(5.0);

        ui.add(Slider::new(&mut spawn_gui.particle_size_min, 0.1..=2.0).text("Particle size min"));
        ui.add(
            Slider::new(
                &mut spawn_gui.particle_size_max,
                spawn_gui.particle_size_min..=2.0,
            )
            .text("Particle size max"),
        );
    }

    fn post_fx_tab(&mut self, gui_ctx: GuiContext, ui: &mut Ui) {
        let gfx_state = &gui_ctx.gfx_state;
        let post_process = gui_ctx.post_process;
        let post_fx = &mut post_process.post_fx;

        post_fx.retain_mut(|fx| {
            fx.create_ui(ui, gfx_state);
            ui.separator();
            !fx.delete()
        });

        ComboBox::from_label("Selected view")
            .selected_text(&post_process.selected_view)
            .show_ui(ui, |ui| {
                for item in post_process.views.iter() {
                    ui.selectable_value(
                        &mut post_process.selected_view,
                        item.tag.to_string(),
                        item.tag.to_string(),
                    );
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            ComboBox::from_id_source("post-fx")
                .selected_text(format!("{:?}", &self.selected_post_fx))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_post_fx, PostFx::Bloom, "Bloom");
                    ui.selectable_value(
                        &mut self.selected_post_fx,
                        PostFx::ColorProcessing,
                        "ColorProcessing",
                    );
                });

            if ui.button("Add Post fx").clicked() {
                match self.selected_post_fx {
                    PostFx::Bloom => {
                        let options = post_process.create_fx_options(gfx_state);
                        let fx = Bloom::new(&options, BloomExport::default());

                        post_process.post_fx.push(Box::new(fx));
                    }
                    PostFx::ColorProcessing => {
                        let options = post_process.create_fx_options(gfx_state);
                        let fx = ColorProcessing::new(&options, ColorProcessingUniform::new());

                        post_process.post_fx.push(Box::new(fx));
                    }
                };
            }
        });

        ui.separator();

        if ui.button("Export").clicked() {
            let to_export: Vec<FxPersistenceType> =
                post_process.post_fx.iter().map(|fx| fx.export()).collect();

            Persistence::write_to_file(to_export, ExportType::PostFx);
            // TODO feedback that export has been successful
        }
    }

    fn spawn_tab(&mut self, gui_ctx: GuiContext, ui: &mut Ui) {
        let GuiContext {
            spawners,
            light_spawner,
            ..
        } = gui_ctx;

        let mut ids: Vec<&str> = spawners.iter().map(|s| s.id.as_str()).collect();
        ids.push(&light_spawner.id);

        egui::ComboBox::from_id_source("sel-spawner")
            .selected_text(&self.selected_spawner_id)
            .show_ui(ui, |ui| {
                for id in ids.into_iter() {
                    ui.selectable_value(
                        &mut self.selected_spawner_id,
                        id.to_owned(),
                        id.to_owned(),
                    );
                }
            });

        let opt_light_spawner = || {
            if light_spawner.id == self.selected_spawner_id {
                Some(light_spawner)
            } else {
                None
            }
        };

        let spawner: Option<&mut SpawnState> = spawners
            .iter_mut()
            .find(|s| s.id == self.selected_spawner_id)
            .or_else(opt_light_spawner);

        if let Some(spawner) = spawner {
            self.create_spawner_menu(ui, &mut spawner.gui, &spawner.id);
        }
    }

    fn create_gui(&mut self, data: GuiContext, ctx: &Context) {
        Window::new("Sparticles settings").show(&ctx, |ui| {
            create_label(ui, &self.fps_text);
            create_label(ui, &self.cpu_time_text);
            create_label(ui, &self.elapsed_text);
            create_label(ui, &self.particle_count_text);

            self.reset_camera = ui.button("Reset camera").clicked();
            ui.add_space(5.0);
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::SpawnSettings, "Spawn settings");
                ui.selectable_value(&mut self.selected_tab, Tab::PostFxSettings, "Post FX");
                ui.selectable_value(&mut self.selected_tab, Tab::AnimationSettings, "Animations");
            });

            ui.separator();
            ui.add_space(5.0);

            match self.selected_tab {
                Tab::SpawnSettings => self.spawn_tab(data, ui),
                Tab::PostFxSettings => self.post_fx_tab(data, ui),
                Tab::AnimationSettings => {}
            };
        });
    }
}

impl AppState {
    pub fn update_gui(&mut self, ctx: &Context, gfx_state: &GfxState) {
        if self.gui.enabled {
            self.update_labels();

            let options = GuiContext {
                spawners: &mut self.spawners,
                light_spawner: &mut self.light_spawner,
                post_process: &mut self.post_process,
                gfx_state,
            };

            self.gui.create_gui(options, ctx);
        }
    }

    fn update_labels(&mut self) {
        let clock = &self.clock;

        if clock.frame() % 20 != 0 {
            return;
        }

        let particle_count_text = self.particle_count_text();
        let gui = &mut self.gui;
        gui.cpu_time_text = clock.cpu_time_text();
        gui.fps_text = clock.fps_text();
        gui.elapsed_text = clock.elapsed_text();
        gui.particle_count_text = particle_count_text;
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
