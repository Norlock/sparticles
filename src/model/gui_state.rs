use super::{EmitterState, GfxState, State};
use crate::{
    animations::ItemAction,
    fx::{bloom::BloomExport, Bloom, ColorProcessing, ColorProcessingUniform, PostProcessState},
    texture::CustomTexture,
    util::Persistence,
};
use egui::{Color32, RichText, Slider, Ui, Window};
use egui_wgpu::wgpu;
use egui_winit::egui::{
    self, epaint::TextureManager, ComboBox, ImageButton, TextureHandle, TextureId, Vec2,
};

use std::path::PathBuf;

pub struct GuiState {
    pub enabled: bool,
    pub reset_camera: bool,

    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,
    texture_paths: Vec<PathBuf>,
    gui_textures: Vec<GuiTextures>,
    selected_tab: Tab,
    selected_post_fx: PostFx,
    selected_texture: usize,
    selected_new_par_anim: usize,
    selected_new_em_anim: usize,
    selected_emitter_id: usize,
}

const CHEVRON_UP_ID: &str = "chevron-up";
const CHEVRON_DOWN_ID: &str = "chevron-down";
const TRASH_ID: &str = "trash";

struct GuiTextures {
    pub tag: String,
    pub texture_id: TextureId,
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
                post_process,
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

            let emitter_txts: Vec<&str> = emitters
                .iter()
                .map(|em| em.id())
                .chain([lights.id()])
                .collect();

            ComboBox::from_id_source("sel-spawner").show_index(
                ui,
                &mut gui.selected_emitter_id,
                emitters.len() + 1,
                |i| emitter_txts[i],
            );

            ui.add_space(5.0);

            ui.separator();

            if ui.button("Export settings").clicked() {
                EmitterState::export(emitters, lights);
                PostProcessState::export(post_process);
            }

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
        idx: usize,
    ) -> Option<&'a mut EmitterState> {
        emitters
            .iter_mut()
            .chain([lights])
            .enumerate()
            .find(|item| item.0 == idx)
            .map(|item| item.1)
    }

    fn emitter_animations_tab(state: &mut State, ui: &mut Ui) {
        if let Some(emitter) = GuiState::selected_emitter(
            &mut state.emitters,
            &mut state.lights,
            state.gui.selected_emitter_id,
        ) {
            let registered_em_anims = &state.registered_em_anims;
            let gui = &mut state.gui;

            emitter.gui_emitter_animations(ui);

            ui.separator();

            ui.horizontal(|ui| {
                let sel_animation = &mut gui.selected_new_em_anim;

                ComboBox::from_id_source("new-particle-animation").show_index(
                    ui,
                    sel_animation,
                    registered_em_anims.len(),
                    |i| registered_em_anims[i].tag(),
                );

                if ui.button("Add animation").clicked() {
                    emitter.push_emitter_animation(
                        registered_em_anims[*sel_animation].create_default(),
                    );
                }
            });
        }
    }

    fn particle_animations_tab(state: &mut State, ui: &mut Ui) {
        let State {
            emitters: e,
            lights: l,
            gui,
            registered_par_anims,
            ..
        } = state;

        if let Some(emitter) = GuiState::selected_emitter(e, l, gui.selected_emitter_id) {
            emitter.gui_particle_animations(ui, gui);

            ui.separator();

            ui.horizontal(|ui| {
                let sel_animation = &mut gui.selected_new_par_anim;

                ComboBox::from_id_source("new-particle-animation").show_index(
                    ui,
                    sel_animation,
                    registered_par_anims.len(),
                    |i| registered_par_anims[i].tag(),
                );

                if ui.button("Add animation").clicked() {
                    emitter.push_particle_animation(
                        registered_par_anims[*sel_animation]
                            .create_default(&state.gfx_state, emitter),
                    );
                }
            });
        }
    }

    fn create_icons(gfx_state: &mut GfxState) -> Vec<GuiTextures> {
        let device = &gfx_state.device;
        let queue = &gfx_state.queue;
        let renderer = &mut gfx_state.renderer;

        let mut textures = Vec::new();

        // Chevron up
        let mut create_tex = |filename: &str, tag: &str| {
            let mut icon_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            icon_path.push("src/assets/icons/");
            icon_path.push(filename);
            let path_str = icon_path
                .to_str()
                .expect(&format!("File doesn't exist: {}", filename));

            let view = CustomTexture::new(device, queue, path_str);

            let texture_id =
                renderer.register_native_texture(device, &view, wgpu::FilterMode::Nearest);

            GuiTextures {
                tag: tag.to_string(),
                texture_id,
            }
        };

        textures.push(create_tex("chevron-up.png", CHEVRON_UP_ID));
        textures.push(create_tex("chevron-down.png", CHEVRON_DOWN_ID));
        textures.push(create_tex("trash.png", TRASH_ID));

        textures
    }

    pub fn new(show_gui: bool, gfx_state: &mut GfxState) -> Self {
        let texture_paths = Persistence::import_textures().unwrap();
        let gui_textures = Self::create_icons(gfx_state);

        //texture_ids.push();

        //let img = egui::Image::new("src/assets/icons/chevron-up.png");
        //egui::Image::new(chevron_up_id, Vec2::new(32., 32.));

        Self {
            enabled: show_gui,
            reset_camera: false,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_tab: Tab::SpawnSettings,
            selected_post_fx: PostFx::Bloom,
            texture_paths,
            selected_emitter_id: 0,
            selected_new_par_anim: 0,
            selected_new_em_anim: 0,
            selected_texture: 0,
            gui_textures,
        }
    }

    pub fn process_gui(state: &mut State) {
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

        if emitters.len() == gui.selected_emitter_id {
            lights.process_gui(None, gfx_state, camera);
        } else if let Some(emitter) = emitters.get_mut(gui.selected_emitter_id) {
            emitter.process_gui(Some(&lights.bind_group_layout), gfx_state, camera);
        }
    }

    fn emitter_settings_tab(state: &mut State, ui: &mut Ui) {
        let gui = &mut state.gui;
        if let Some(emitter) = GuiState::selected_emitter(
            &mut state.emitters,
            &mut state.lights,
            gui.selected_emitter_id,
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

            ComboBox::from_label("select-texture").show_index(
                ui,
                &mut gui.selected_texture,
                gui.texture_paths.len(),
                |i| gui.texture_paths[i].rich_text(),
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
    }

    pub fn create_title(ui: &mut Ui, str: &str) {
        ui.label(RichText::new(str).color(Color32::WHITE).size(16.0));
        ui.add_space(5.0);
    }

    pub fn create_anim_header(&self, ui: &mut Ui, selected_action: &mut ItemAction, title: &str) {
        ui.horizontal_top(|ui| {
            GuiState::create_title(ui, title);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let trash_id = self
                    .gui_textures
                    .iter()
                    .find(|icon| icon.tag == TRASH_ID)
                    .map(|icon| icon.texture_id)
                    .expect("Texture doesn't exist");

                if ui.add(ImageButton::new(trash_id, [16., 16.])).clicked() {
                    *selected_action = ItemAction::Delete;
                };

                let up_id = self
                    .gui_textures
                    .iter()
                    .find(|icon| icon.tag == CHEVRON_UP_ID)
                    .map(|icon| icon.texture_id)
                    .expect("Texture doesn't exist");

                if ui.add(ImageButton::new(up_id, [16., 16.])).clicked() {
                    *selected_action = ItemAction::MoveUp;
                };

                let down_id = self
                    .gui_textures
                    .iter()
                    .find(|icon| icon.tag == CHEVRON_DOWN_ID)
                    .map(|icon| icon.texture_id)
                    .expect("Texture doesn't exist");

                if ui.add(ImageButton::new(down_id, [16., 16.])).clicked() {
                    *selected_action = ItemAction::MoveDown;
                };
            });
        });
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

pub trait IntoRichText {
    fn rich_text(&self) -> RichText;
}

impl IntoRichText for PathBuf {
    fn rich_text(&self) -> RichText {
        RichText::new(self.file_name().unwrap().to_str().unwrap())
    }
}
