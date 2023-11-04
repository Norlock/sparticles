use super::{events::ViewIOEvent, EmitterState, GfxState, State};
use crate::{
    fx::{post_process::CreateFxOptions, PostProcessState},
    texture::IconTexture,
    util::ListAction,
    util::Persistence,
};
use egui::{Color32, RichText, Slider, Ui, Window};
use egui_wgpu::wgpu;
use egui_winit::{
    egui::{
        self, load::SizedTexture, CollapsingHeader, ComboBox, DisplayEvent, ImageButton, Margin,
        SetEvent, TextureId,
    },
    winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
};
use std::{collections::HashMap, path::PathBuf};
use wgpu_profiler::GpuTimerScopeResult;

pub struct GuiState {
    pub enabled: bool,
    pub new_emitter_tag: String,
    pub profiling_results: Vec<GpuTimerScopeResult>,

    fps_text: String,
    cpu_time_text: String,
    elapsed_text: String,
    particle_count_text: String,
    texture_paths: Vec<PathBuf>,
    icon_textures: HashMap<String, TextureId>,
    selected_tab: Tab,
    selected_texture: usize,
    selected_new_par_anim: usize,
    selected_new_em_anim: usize,
    selected_new_post_fx: usize,
    selected_emitter_id: usize,

    performance_event: Option<DisplayEvent>,
    display_event: Option<DisplayEvent>,
}

const CHEVRON_UP_ID: &str = "chevron-up";
const CHEVRON_DOWN_ID: &str = "chevron-down";
const TRASH_ID: &str = "trash";

#[derive(PartialEq)]
enum Tab {
    EmitterSettings,
    PostFxSettings,
    ParticleAnimations,
    EmitterAnimations,
}

impl GuiState {
    pub fn process_input(state: &mut State, input: &KeyboardInput, shift_pressed: bool) -> bool {
        let gui = &mut state.gui;
        let events = &mut state.events;

        if !gui.enabled || input.state == ElementState::Pressed {
            return false;
        }

        let keycode = input.virtual_keycode.unwrap_or(VirtualKeyCode::Return);

        match keycode {
            VirtualKeyCode::T if shift_pressed => {
                events.set_io_view(ViewIOEvent::Subtract);
            }
            VirtualKeyCode::T if !shift_pressed => {
                events.set_io_view(ViewIOEvent::Add);
            }
            VirtualKeyCode::C => gui.display_event.set(DisplayEvent::ToggleCollapse),
            VirtualKeyCode::Key1 => gui.selected_tab = Tab::EmitterSettings,
            VirtualKeyCode::Key2 => gui.selected_tab = Tab::PostFxSettings,
            VirtualKeyCode::Key3 => gui.selected_tab = Tab::ParticleAnimations,
            VirtualKeyCode::Key4 => gui.selected_tab = Tab::EmitterAnimations,
            VirtualKeyCode::P => gui.performance_event.set(DisplayEvent::ToggleCollapse),
            _ => return false,
        }

        true
    }

    pub fn update_gui(state: &mut State) {
        if !state.gui.enabled {
            return;
        }

        let margin_size = 10.;

        Window::new("Sparticles settings")
            .vscroll(true)
            .frame(egui::Frame {
                fill: Color32::from_rgb(0, 30, 0),
                inner_margin: Margin {
                    top: margin_size,
                    left: margin_size,
                    right: margin_size,
                    bottom: margin_size,
                },
                ..Default::default()
            })
            .default_height(800.)
            .display_event(&mut state.gui.display_event)
            .show(&state.gfx_state.ctx.clone(), |ui| {
                let State {
                    clock,
                    lights,
                    emitters,
                    gui,
                    gfx_state,
                    post_process,
                    events,
                    ..
                } = state;

                // Update gui info
                if clock.frame() % 20 == 0 {
                    let particle_count: u64 = lights.particle_count()
                        + emitters.iter().map(|s| s.particle_count()).sum::<u64>();

                    gui.cpu_time_text = clock.frame_time_text();
                    gui.fps_text = clock.fps_text();
                    gui.elapsed_text = clock.elapsed_text();
                    gui.particle_count_text = format!("Particle count: {}", particle_count);

                    let prof = &mut gfx_state.profiler;
                    let queue = &gfx_state.queue;

                    if let Some(res) = prof.process_finished_frame(queue.get_timestamp_period()) {
                        gui.profiling_results = res;
                    }
                }

                create_label(ui, &gui.fps_text);
                create_label(ui, &gui.cpu_time_text);
                create_label(ui, &gui.elapsed_text);
                create_label(ui, &gui.particle_count_text);
                ui.separator();

                CollapsingHeader::new("Performance")
                    .id_source("total")
                    .display(&mut gui.performance_event)
                    .show(ui, |ui| {
                        let mut total: f64 = 0.;
                        Self::display_performance(ui, &mut total, &mut gui.profiling_results);
                        create_label(
                            ui,
                            format!("{} - {:.3}μs", "Total GPU time", total * 1_000_000.),
                        );
                    });

                ui.separator();

                let emitter_txts: Vec<&str> = emitters
                    .iter()
                    .map(|em| em.id())
                    .chain([lights.id()])
                    .collect();

                ui.horizontal(|ui| {
                    ComboBox::from_id_source("select-emitter").show_index(
                        ui,
                        &mut gui.selected_emitter_id,
                        emitter_txts.len(),
                        |i| emitter_txts[i],
                    );

                    ui.separator();

                    ui.label("New emitter tag:");
                    ui.add(
                        egui::TextEdit::singleline(&mut gui.new_emitter_tag).desired_width(100.),
                    );

                    let is_enabled = 3 <= gui.new_emitter_tag.len()
                        && emitters.iter().all(|em| em.id() != &gui.new_emitter_tag)
                        && lights.id() != &gui.new_emitter_tag;

                    if ui
                        .add_enabled(is_enabled, egui::Button::new("Add emitter"))
                        .clicked()
                    {
                        events.set_create_emitter(gui.new_emitter_tag.to_string());
                        gui.new_emitter_tag = "".to_string();
                    }
                });

                ui.add_space(5.0);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Export settings").clicked() {
                        EmitterState::export(emitters, lights);
                        PostProcessState::export(post_process);
                    }

                    if ui.button("Reset camera").clicked() {
                        events.set_reset_camera();
                    }

                    let emitter =
                        GuiState::selected_emitter(emitters, lights, gui.selected_emitter_id)
                            .expect("Expects a selected emitter");

                    if !emitter.is_light && ui.button("Remove emitter").clicked() {
                        let tag = emitter.id().to_string();
                        events.set_delete_emitter(tag);
                        gui.selected_emitter_id = 0;
                    }
                });

                ui.add_space(5.0);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut gui.selected_tab,
                        Tab::EmitterSettings,
                        "Spawn settings",
                    );
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
                    Tab::EmitterSettings => GuiState::emitter_settings_tab(state, ui),
                    Tab::PostFxSettings => GuiState::post_fx_tab(state, ui),
                    Tab::ParticleAnimations => GuiState::particle_animations_tab(state, ui),
                    Tab::EmitterAnimations => GuiState::emitter_animations_tab(state, ui),
                };
            });
    }

    fn display_performance(ui: &mut Ui, total_time: &mut f64, results: &[GpuTimerScopeResult]) {
        for scope in results.iter() {
            let time = scope.time.end - scope.time.start;
            *total_time += time;
            let display_value = format!("{} - {:.3}μs", scope.label, time * 1_000_000.);

            create_label(ui, display_value);

            if !scope.nested_scopes.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(5.);
                    CollapsingHeader::new("-- details --")
                        .id_source(&scope.label)
                        .show(ui, |ui| {
                            Self::display_performance(ui, total_time, &scope.nested_scopes)
                        });
                });
            }
        }
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

            emitter.gui_emitter_animations(ui, gui);

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

    fn create_icons(gfx_state: &mut GfxState) -> HashMap<String, TextureId> {
        let device = &gfx_state.device;
        let queue = &gfx_state.queue;
        let renderer = &mut gfx_state.renderer;

        let mut textures = HashMap::new();

        let mut create_tex = |filename: &str, tag: &str| {
            let mut icon_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            icon_path.push("src/assets/icons/");
            icon_path.push(filename);
            let path_str = icon_path
                .to_str()
                .unwrap_or_else(|| panic!("File doesn't exist: {}", filename));

            let view = IconTexture::create_view(device, queue, path_str);

            let texture_id =
                renderer.register_native_texture(device, &view, wgpu::FilterMode::Nearest);

            textures.insert(tag.to_string(), texture_id);
        };

        create_tex("chevron-up.png", CHEVRON_UP_ID);
        create_tex("chevron-down.png", CHEVRON_DOWN_ID);
        create_tex("trash.png", TRASH_ID);

        textures
    }

    pub fn new(show_gui: bool, gfx_state: &mut GfxState) -> Self {
        let texture_paths = Persistence::import_textures().unwrap();
        let icon_textures = Self::create_icons(gfx_state);

        Self {
            enabled: show_gui,
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_tab: Tab::EmitterSettings,
            texture_paths,
            selected_emitter_id: 0,
            selected_new_par_anim: 0,
            selected_new_em_anim: 0,
            selected_new_post_fx: 0,
            selected_texture: 0,
            icon_textures,
            new_emitter_tag: String::from(""),
            profiling_results: Vec::new(),
            display_event: None,
            performance_event: None,
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
            create_label(ui, "Box position");

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut emitter_gui.box_position.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut emitter_gui.box_position.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut emitter_gui.box_position.z).speed(0.1));
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
                Slider::new(&mut emitter_gui.particle_size_min, 0.01..=2.0)
                    .text("Particle size min"),
            );
            ui.add(
                Slider::new(
                    &mut emitter_gui.particle_size_max,
                    emitter_gui.particle_size_min..=2.0,
                )
                .text("Particle size max"),
            );

            let combo_texture = ComboBox::from_label("Select texture").show_index(
                ui,
                &mut gui.selected_texture,
                gui.texture_paths.len(),
                |i| gui.texture_paths[i].rich_text(),
            );

            if combo_texture.changed() {
                emitter.update_diffuse(
                    &state.gfx_state,
                    &mut gui.texture_paths[gui.selected_texture],
                );
            };
        }
    }

    fn post_fx_tab(state: &mut State, ui: &mut Ui) {
        let State {
            post_process,
            gui,
            registered_post_fx,
            gfx_state,
            events,
            ..
        } = state;

        let effects = &mut post_process.effects;
        for fx in effects.iter_mut() {
            fx.create_ui(ui, gui);
            ui.separator();
        }

        ui.separator();

        ui.horizontal(|ui| {
            let sel_post_fx = &mut gui.selected_new_post_fx;

            ComboBox::from_id_source("new-post-fx").show_index(
                ui,
                sel_post_fx,
                registered_post_fx.len(),
                |i| registered_post_fx[i].tag(),
            );

            if ui.button("Add post fx").clicked() {
                effects.push(
                    registered_post_fx[*sel_post_fx].create_default(&CreateFxOptions {
                        fx_state: &post_process.fx_state,
                        gfx_state,
                    }),
                );
            }
        });

        ui.add_space(10.);
        ui.horizontal(|ui| {
            let mut tex_output = post_process.io_uniform.out_idx as usize;

            if egui::ComboBox::from_id_source("select-tex-output")
                .selected_text("Select texture output")
                .show_index(ui, &mut tex_output, 16, |i| {
                    format!("Texture output: {}", i)
                })
                .changed()
            {
                events.set_io_view(ViewIOEvent::Idx(tex_output as u32))
            }
        });
    }

    pub fn create_title(ui: &mut Ui, str: &str) {
        ui.label(RichText::new(str).color(Color32::WHITE).size(16.0));
        ui.add_space(5.0);
    }

    /// Creates list item header
    pub fn create_li_header(&self, ui: &mut Ui, title: &str) -> ListAction {
        let mut selected_action = ListAction::None;

        ui.horizontal_top(|ui| {
            GuiState::create_title(ui, title);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let trash_id = self
                    .icon_textures
                    .get(TRASH_ID)
                    .expect("Trash icon doesn't exist");

                let trash_img = SizedTexture::new(*trash_id, [16., 16.]);
                if ui.add(ImageButton::new(trash_img)).clicked() {
                    selected_action = ListAction::Delete;
                };

                let up_id = self
                    .icon_textures
                    .get(CHEVRON_UP_ID)
                    .expect("Chevron up icon doesn't exist");

                let up_img = SizedTexture::new(*up_id, [16., 16.]);
                if ui.add(ImageButton::new(up_img)).clicked() {
                    selected_action = ListAction::MoveUp;
                };

                let down_id = self
                    .icon_textures
                    .get(CHEVRON_DOWN_ID)
                    .expect("Chevron down icon doesn't exist");

                let down_img = SizedTexture::new(*down_id, [16., 16.]);
                if ui.add(ImageButton::new(down_img)).clicked() {
                    selected_action = ListAction::MoveDown;
                };
            });
        });

        selected_action
    }

    pub fn create_degree_slider(ui: &mut Ui, val: &mut f32, str: &str) {
        ui.add(Slider::new(val, 0.0..=360.).text(str));
    }
}

fn create_label(ui: &mut Ui, text: impl Into<String>) {
    ui.label(RichText::new(text).color(Color32::WHITE));
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
