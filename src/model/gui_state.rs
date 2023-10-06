use super::{EmitterState, State};
use crate::{
    fx::{bloom::BloomExport, Bloom, ColorProcessing, ColorProcessingUniform, FxPersistenceType},
    traits::RegisterParticleAnimation,
    util::{persistence::ExportType, Persistence},
};
use egui::{Color32, RichText, Slider, Ui, Window};
use egui_wgpu::wgpu;
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
    selected_new_particle_animation: Box<dyn RegisterParticleAnimation>,
}

#[derive(PartialEq)]
enum Tab {
    SpawnSettings,
    PostFxSettings,
    ParticleAnimations,
    EmitterAnimations,
}

#[derive(PartialEq, Debug)]
enum PostFx {
    Bloom,
    ColorProcessing,
}

impl GuiState {
    pub fn update_gui(state: &mut State) {
        if !state.gui.enabled {
            return;
        }

        Window::new("Sparticles settings").show(&state.gfx_state.ctx.clone(), |ui| {
            let State {
                clock,
                emitters,
                lights,
                gui,
                ..
            } = state;

            // Update gui info
            if clock.frame() % 20 == 0 {
                let particle_count: u64 = lights.particle_count()
                    + emitters.iter().map(|s| s.particle_count()).sum::<u64>();

                gui.cpu_time_text = clock.cpu_time_text();
                gui.fps_text = clock.fps_text();
                gui.elapsed_text = clock.elapsed_text();
                gui.particle_count_text = format!("Particle count: {}", particle_count);
            }

            // Set labels
            create_label(ui, &gui.fps_text);
            create_label(ui, &gui.cpu_time_text);
            create_label(ui, &gui.elapsed_text);
            create_label(ui, &gui.particle_count_text);

            gui.reset_camera = ui.button("Reset camera").clicked();
            ui.add_space(5.0);

            let ids: Vec<&str> = emitters
                .iter()
                .map(|em| em.id())
                .chain(vec![lights.id()])
                .collect();

            ComboBox::from_id_source("sel-spawner")
                .selected_text(&gui.selected_spawner_id)
                .show_ui(ui, |ui| {
                    for id in ids.into_iter() {
                        ui.selectable_value(
                            &mut gui.selected_spawner_id,
                            id.to_owned(),
                            id.to_owned(),
                        );
                    }
                });

            ui.add_space(5.0);
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut gui.selected_tab, Tab::SpawnSettings, "Spawn settings");
                ui.selectable_value(&mut gui.selected_tab, Tab::PostFxSettings, "Post FX");
                ui.selectable_value(
                    &mut gui.selected_tab,
                    Tab::ParticleAnimations,
                    "Particle animations",
                );
                ui.selectable_value(
                    &mut gui.selected_tab,
                    Tab::EmitterAnimations,
                    "Emitter animations",
                );
            });

            ui.separator();

            match gui.selected_tab {
                Tab::SpawnSettings => GuiState::emitter_settings_tab(state, ui),
                Tab::PostFxSettings => GuiState::post_fx_tab(state, ui),
                Tab::ParticleAnimations => GuiState::particle_animations_tab(state, ui),
                Tab::EmitterAnimations => GuiState::emitter_animations_tab(state, ui),
            };
        });
    }

    pub fn selected_emitter<'a>(
        emitters: &'a mut [EmitterState],
        lights: &'a mut EmitterState,
        id: &'a str,
    ) -> Option<&'a mut EmitterState> {
        emitters
            .iter_mut()
            .find(|emitter| emitter.id() == id)
            .or_else(|| (lights.id() == id).then_some(lights))
    }

    fn emitter_animations_tab(state: &mut State, ui: &mut Ui) {
        if let Some(emitter) = GuiState::selected_emitter(
            &mut state.emitters,
            &mut state.lights,
            &state.gui.selected_spawner_id,
        ) {
            emitter.gui_emitter_animations(ui);
        }
    }

    fn particle_animations_tab(state: &mut State, ui: &mut Ui) {
        let State {
            emitters: e,
            lights: l,
            gui,
            registered_particle_animations,
            ..
        } = state;

        if let Some(emitter) = GuiState::selected_emitter(e, l, &gui.selected_spawner_id) {
            emitter.gui_particle_animations(ui);

            let sel_animation = &mut gui.selected_new_particle_animation;

            ui.separator();

            ui.horizontal(|ui| {
                ComboBox::from_id_source("new-particle-animation")
                    .selected_text(sel_animation.tag())
                    .show_ui(ui, |ui| {
                        for anim in registered_particle_animations.iter_mut() {
                            ui.selectable_value(sel_animation, anim.dyn_clone(), anim.tag());
                        }
                    });

                if ui.button("Add animation").clicked() {
                    let anim = sel_animation.create_default(&state.gfx_state, emitter);
                    emitter.push_particle_animation(anim);
                }
            });
        }
    }

    pub fn new(
        spawners: &[EmitterState],
        all_animations: &[Box<dyn RegisterParticleAnimation>],
        show_gui: bool,
    ) -> Self {
        let spawner = spawners.first();

        let selected_id = spawner.map_or("".to_owned(), |s| s.id().to_owned());
        let selected_new_particle_animation = all_animations[0].dyn_clone();

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
            selected_new_particle_animation,
        }
    }

    pub fn handle_gui(state: &mut State) {
        let State {
            camera,
            lights,
            emitters,
            gui,
            gfx_state,
            ..
        } = state;

        if !gui.enabled {
            return;
        }

        if gui.reset_camera {
            camera.pitch = 0.;
            camera.yaw = 0.;
            camera.position = glam::Vec3::new(0., 0., 10.);
            camera.view_dir = glam::Vec3::new(0., 0., -10.);
        }

        let update_emitter = |emitter: &mut EmitterState,
                              layout: Option<&wgpu::BindGroupLayout>| {
            emitter.uniform.handle_gui(&emitter.gui);

            if emitter.gui.recreate {
                emitter.recreate_spawner(gfx_state, layout, camera);
            }
        };

        if lights.id() == gui.selected_spawner_id {
            update_emitter(lights, None);
        } else if let Some(emitter) = emitters
            .iter_mut()
            .find(|emitter| emitter.id() == gui.selected_spawner_id)
        {
            update_emitter(emitter, Some(&lights.bind_group_layout));
        }
    }

    fn emitter_settings_tab(state: &mut State, ui: &mut Ui) {
        if let Some(emitter) = GuiState::selected_emitter(
            &mut state.emitters,
            &mut state.lights,
            &state.gui.selected_spawner_id,
        ) {
            let emitter_gui = &mut emitter.gui;

            ui.add_space(5.0);

            Self::create_degree_slider(ui, &mut emitter_gui.box_rotation_deg.x, "Box yaw");
            Self::create_degree_slider(ui, &mut emitter_gui.box_rotation_deg.y, "Box pitch");
            Self::create_degree_slider(ui, &mut emitter_gui.box_rotation_deg.z, "Box roll");

            Self::create_degree_slider(ui, &mut emitter_gui.diff_width_deg, "Diffusion width");
            Self::create_degree_slider(ui, &mut emitter_gui.diff_depth_deg, "Diffusion depth");

            ui.add_space(5.0);
            create_label(ui, "Box dimensions (w, h, d)");

            ui.horizontal(|ui| {
                create_drag_value(ui, &mut emitter_gui.box_dimensions.x);
                create_drag_value(ui, &mut emitter_gui.box_dimensions.y);
                create_drag_value(ui, &mut emitter_gui.box_dimensions.z);
            });

            ui.add_space(5.0);
            ui.add(
                Slider::new(&mut emitter_gui.particle_speed_min, 0.0..=50.0)
                    .text("Particle emit speed min"),
            );
            ui.add(
                Slider::new(
                    &mut emitter_gui.particle_speed_max,
                    emitter_gui.particle_speed_min..=50.0,
                )
                .text("Particle emit speed max"),
            );
            ui.add_space(5.0);
            create_label(ui, "Spawn itemings");

            ui.add(
                egui::Slider::new(&mut emitter_gui.particle_lifetime_sec, 1.0..=40.0)
                    .drag_value_speed(0.)
                    .max_decimals(1)
                    .step_by(0.1)
                    .text("Particle lifetime (sec)"),
            );

            ui.add(
                egui::Slider::new(&mut emitter_gui.spawn_delay_sec, 0.1..=20.0)
                    .drag_value_speed(0.)
                    .max_decimals(1)
                    .step_by(0.1)
                    .text("Spawn delay (sec)"),
            );

            ui.add(egui::Slider::new(&mut emitter_gui.spawn_count, 1..=100).text("Spawn count"));

            ui.add_space(5.0);

            emitter_gui.recreate = ui.button("Update spawn settings").clicked();

            ui.add_space(5.0);

            create_label(ui, "Particle settings");

            ui.add_space(5.0);

            ui.add(
                Slider::new(&mut emitter_gui.particle_size_min, 0.1..=2.0)
                    .text("Particle size min"),
            );
            ui.add(
                Slider::new(
                    &mut emitter_gui.particle_size_max,
                    emitter_gui.particle_size_min..=2.0,
                )
                .text("Particle size max"),
            );
        }
    }

    fn post_fx_tab(state: &mut State, ui: &mut Ui) {
        let State {
            gfx_state,
            post_process,
            gui,
            ..
        } = state;
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
                .selected_text(format!("{:?}", &gui.selected_post_fx))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut gui.selected_post_fx, PostFx::Bloom, "Bloom");
                    ui.selectable_value(
                        &mut gui.selected_post_fx,
                        PostFx::ColorProcessing,
                        "ColorProcessing",
                    );
                });

            if ui.button("Add Post fx").clicked() {
                match gui.selected_post_fx {
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

    pub fn create_title(ui: &mut Ui, str: &str) {
        ui.label(RichText::new(str).color(Color32::WHITE).size(16.0));
        ui.add_space(5.0);
    }

    pub fn create_degree_slider(ui: &mut Ui, val: &mut f32, str: &str) {
        ui.add(Slider::new(val, 0.0..=360.).text(str));
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
