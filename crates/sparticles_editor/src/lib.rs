use crate::widgets::EditorWidgets;
use sparticles_app::{
    animations::{
        color_animation::COLOR_ANIM_WIDGETS, force_animation::ForceAnimation,
        gravity_animation::GravityAnimation, ColorAnimation, StrayAnimation,
    },
    fx::{FxOptions, PostProcessState},
    gui::egui::{load::SizedTexture, *},
    gui::{
        egui::{
            self,
            color_picker::{color_edit_button_rgba, Alpha},
        },
        winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
    },
    model::{
        camera::TonemapType, emitter_state::RecreateEmitterOptions, events::ViewIOEvent,
        EmitterSettings, EmitterState, EmitterType, Events, GfxState, State,
    },
    profiler::GpuTimerScopeResult,
    texture::IconTexture,
    traits::{ParticleAnimation, Splitting, WidgetBuilder},
    util::ListAction,
    util::Persistence,
    wgpu::{self, CommandEncoder},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::PathBuf,
};

pub mod menu;
pub mod widgets;

pub type PAWidgetPtr = Box<dyn Fn(&mut EditorData, &mut Box<dyn ParticleAnimation>, &mut Ui)>;

pub struct Editor {
    pub data: EditorData,

    /// Particle animation Widgets
    pub pa_widgets: HashMap<TypeId, PAWidgetPtr>,
}

pub struct EditorData {
    new_emitter_tag: String,
    profiling_results: Vec<GpuTimerScopeResult>,
    pub selected_emitter_idx: usize,
    selected_menu_idx: usize,

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

    //performance_event: Option<DisplayEvent>,
    //display_event: Option<DisplayEvent>,
    pub emitter_settings: Option<EmitterSettings>,
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

const WINDOW_MARGIN: f32 = 10.;

impl WidgetBuilder for Editor {
    fn draw_ui(&mut self, state: &mut State, encoder: &mut wgpu::CommandEncoder) -> Events {
        let mut events = Events::default();
        self.update_gui(state, &mut events, encoder);
        events
    }

    fn id(&self) -> &'static str {
        "editor"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn draw_widget(&mut self, anim: &mut Box<dyn ParticleAnimation>, ui: &mut Ui) {
        let type_id = get_type_id(anim.as_any());

        if let Some(widget) = self.pa_widgets.get_mut(&type_id) {
            widget(&mut self.data, anim, ui);
        } else {
            println!("widget not found");
        }
    }
}

/// Weird trick to avoid the borrow checker nonsense
fn get_type_id<T: ?Sized + Any>(s: &T) -> TypeId {
    s.type_id()
}

impl Editor {
    pub fn process_input(
        &mut self,
        state: &mut State,
        input: &KeyboardInput,
        events: &mut Events,
        shift_pressed: bool,
    ) -> bool {
        if input.state == ElementState::Pressed {
            return false;
        }

        let data = &mut self.data;

        let keycode = input.virtual_keycode.unwrap_or(VirtualKeyCode::Return);

        match keycode {
            VirtualKeyCode::T if shift_pressed => {
                events.io_view = Some(ViewIOEvent::Subtract);
            }
            VirtualKeyCode::T if !shift_pressed => {
                events.io_view = Some(ViewIOEvent::Add);
            }
            VirtualKeyCode::Key1 => data.selected_tab = Tab::EmitterSettings,
            VirtualKeyCode::Key2 => data.selected_tab = Tab::PostFxSettings,
            VirtualKeyCode::Key3 => data.selected_tab = Tab::ParticleAnimations,
            VirtualKeyCode::Key4 => data.selected_tab = Tab::EmitterAnimations,
            //VirtualKeyCode::C => gui.display_event.set(DisplayEvent::ToggleCollapse),
            //VirtualKeyCode::P => gui.performance_event.set(DisplayEvent::ToggleCollapse),
            VirtualKeyCode::F => events.toggle_game_state(),
            _ => return false,
        }

        true
    }

    pub fn update_gui(
        &mut self,
        state: &mut State,
        events: &mut Events,
        encoder: &mut CommandEncoder,
    ) {
        Window::new("Menu").show(state.egui_ctx(), |ui| {
            //let gui = &mut state.gui;

            //ComboBox::from_id_source("select-emitter").show_index(
            //ui,
            //&mut gui.selected_emitter_idx,
            //emitters.len(),
            //|i| emitters[i].id(),
            //);
        });

        Window::new("Sparticles settings")
            .vscroll(true)
            .frame(egui::Frame {
                fill: state.gfx.ctx.style().visuals.window_fill,

                inner_margin: Margin {
                    top: WINDOW_MARGIN,
                    left: WINDOW_MARGIN,
                    right: WINDOW_MARGIN,
                    bottom: WINDOW_MARGIN,
                },
                ..Default::default()
            })
            .default_height(800.)
            .show(&state.egui_ctx().clone(), |ui| {
                let State {
                    clock,
                    emitters,
                    gfx,
                    post_process,
                    ..
                } = state;
                let data = &mut self.data;

                // Update gui info
                if clock.frame() % 20 == 0 && events.play() {
                    let count: u64 = emitters.iter().map(|s| s.particle_count()).sum();

                    data.cpu_time_text = clock.frame_time_text();
                    data.fps_text = clock.fps_text();
                    data.elapsed_text = clock.elapsed_text();
                    data.particle_count_text = format!("Particle count: {}", count);

                    let prof = &mut gfx.profiler;
                    let queue = &gfx.queue;

                    if let Some(res) = prof.process_finished_frame(queue.get_timestamp_period()) {
                        data.profiling_results = res;
                    }
                }

                create_label(ui, &data.fps_text);
                create_label(ui, &data.cpu_time_text);
                create_label(ui, &data.elapsed_text);
                create_label(ui, &data.particle_count_text);
                ui.separator();

                CollapsingHeader::new("Performance")
                    .id_source("total")
                    .show(ui, |ui| {
                        let total = Self::display_performance(ui, &mut data.profiling_results);
                        create_label(
                            ui,
                            format!("{} - {:.3}μs", "Total GPU time", total * 1_000_000.),
                        );
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    ComboBox::from_id_source("select-emitter").show_index(
                        ui,
                        &mut data.selected_emitter_idx,
                        emitters.len(),
                        |i| emitters[i].id(),
                    );

                    ui.separator();

                    ui.label("New emitter tag:");
                    ui.add(
                        egui::TextEdit::singleline(&mut data.new_emitter_tag).desired_width(100.),
                    );

                    let is_enabled = 3 <= data.new_emitter_tag.len()
                        && emitters.iter().all(|em| em.id() != &data.new_emitter_tag);

                    if ui
                        .add_enabled(is_enabled, egui::Button::new("Add emitter"))
                        .clicked()
                    {
                        events.create_emitter = Some(data.new_emitter_tag.to_string());
                        data.new_emitter_tag = "".to_string();
                    }
                });

                ui.add_space(5.0);

                ui.separator();

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("Export settings").clicked() {
                        EmitterState::export(emitters);
                        PostProcessState::export(post_process);
                    }

                    if ui.button("Reset camera").clicked() {
                        events.reset_camera = true;
                    }

                    if ui.button("Toggle pause").clicked() {
                        events.toggle_game_state();
                    }

                    // TODO via events
                    ComboBox::from_label("tonemapping")
                        .selected_text(state.camera.tonemap_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut state.camera.tonemap_type,
                                TonemapType::AcesNarkowicz,
                                TonemapType::AcesNarkowicz,
                            );
                            ui.selectable_value(
                                &mut state.camera.tonemap_type,
                                TonemapType::AcesHill,
                                TonemapType::AcesHill,
                            );
                            ui.selectable_value(
                                &mut state.camera.tonemap_type,
                                TonemapType::Uchimura,
                                TonemapType::Uchimura,
                            );
                            ui.selectable_value(
                                &mut state.camera.tonemap_type,
                                TonemapType::Lottes,
                                TonemapType::Lottes,
                            );
                        });

                    let emitter = &emitters[data.selected_emitter_idx];

                    if !emitter.is_light && ui.button("Remove emitter").clicked() {
                        let id = emitter.id().to_string();
                        events.delete_emitter = Some(id);
                        data.selected_emitter_idx = 0;
                    }
                });

                ui.add_space(5.0);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut data.selected_tab,
                        Tab::EmitterSettings,
                        "Spawn settings",
                    );
                    ui.selectable_value(&mut data.selected_tab, Tab::PostFxSettings, "Post FX");
                    ui.selectable_value(
                        &mut data.selected_tab,
                        Tab::ParticleAnimations,
                        "Particle animations",
                    );
                    ui.selectable_value(
                        &mut data.selected_tab,
                        Tab::EmitterAnimations,
                        "Emitter animations",
                    );
                });

                ui.separator();

                match data.selected_tab {
                    Tab::EmitterSettings => self.emitter_settings_tab(state, ui, encoder),
                    Tab::PostFxSettings => self.post_fx_tab(state, ui, events),
                    Tab::ParticleAnimations => self.particle_animations_tab(state, ui),
                    Tab::EmitterAnimations => self.emitter_animations_tab(state, ui),
                };
            });
    }

    fn display_performance(ui: &mut Ui, results: &[GpuTimerScopeResult]) -> f64 {
        let mut total_time = 0.;

        for scope in results.iter() {
            let time = scope.time.end - scope.time.start;
            total_time += time;
            let display_value = format!("{} - {:.3}μs", scope.label, time * 1_000_000.);

            create_label(ui, display_value);

            if !scope.nested_scopes.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(5.);
                    CollapsingHeader::new("-- details --")
                        .id_source(&scope.label)
                        .show(ui, |ui| Self::display_performance(ui, &scope.nested_scopes));
                });
            }
        }

        total_time
    }

    fn emitter_animations_tab(&mut self, state: &mut State, ui: &mut Ui) {
        let emitter = &mut state.emitters[self.data.selected_emitter_idx];
        let registered_em_anims = &state.registry_em_anims;

        self.ui_emitter_animations(emitter, ui);

        let data = &mut self.data;

        ui.separator();

        ui.horizontal(|ui| {
            let sel_animation = &mut data.selected_new_em_anim;

            ComboBox::from_id_source("new-particle-animation").show_index(
                ui,
                sel_animation,
                registered_em_anims.len(),
                |i| registered_em_anims[i].tag(),
            );

            if ui.button("Add animation").clicked() {
                emitter
                    .push_emitter_animation(registered_em_anims[*sel_animation].create_default());
            }
        });
    }

    fn particle_animations_tab(&mut self, state: &mut State, ui: &mut Ui) {
        let State {
            emitters,
            registry_par_anims,
            ..
        } = state;

        let emitter = &mut emitters[self.data.selected_emitter_idx];
        self.ui_particle_animations(emitter, ui);
        let data = &mut self.data;

        ui.separator();

        ui.horizontal(|ui| {
            let sel_animation = &mut data.selected_new_par_anim;

            ComboBox::from_id_source("new-particle-animation").show_index(
                ui,
                sel_animation,
                registry_par_anims.len(),
                |i| registry_par_anims[i].tag(),
            );

            if ui.button("Add animation").clicked() {
                emitter.push_particle_animation(
                    registry_par_anims[*sel_animation].create_default(&state.gfx, emitter),
                );
            }
        });
    }

    pub fn ui_emitter_animations(&mut self, emitter: &mut EmitterState, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in emitter.emitter_animations.iter_mut() {
                    ui.group(|ui| {
                        //anim.create_ui(ui, gui_state);
                        // TODOOOOOO
                    });
                }
            });
    }

    pub fn ui_particle_animations(&mut self, emitter: &mut EmitterState, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in emitter.particle_animations.iter_mut() {
                    ui.group(|ui| {
                        //let dc = anim.type_id();
                        self.draw_widget(anim, ui);
                        //anim.
                        //anim.create_ui(ui, gui_state);
                        //anim.draw(ui, self);
                        //self.draw_widget(ui, anim);
                        //anim.
                    });
                }
            });
    }

    fn create_icons(gfx_state: &mut GfxState) -> HashMap<String, TextureId> {
        let device = &gfx_state.device;
        let queue = &gfx_state.queue;
        let renderer = &mut gfx_state.renderer;

        let mut textures = HashMap::new();

        let mut create_tex = |filename: &str, tag: &str| {
            // TODO weer goed zetten
            let mut icon_path = PathBuf::from(
                "/home/norlock/Projects/sparticles/crates/sparticles_app/src/assets/icons",
            );
            //icon_path.push("crates/sparticles_app/src/assets/icons/");
            println!("{:?}", icon_path.to_str().unwrap());
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

    pub fn new(gfx_state: &mut GfxState) -> Self {
        let texture_paths = Persistence::import_textures().unwrap();
        let icon_textures = Self::create_icons(gfx_state);
        let mut pa_widgets: HashMap<TypeId, PAWidgetPtr> = HashMap::new();

        pa_widgets.insert(
            TypeId::of::<ColorAnimation>(),
            Box::new(EditorWidgets::color_anim),
        );

        pa_widgets.insert(
            TypeId::of::<ForceAnimation>(),
            Box::new(EditorWidgets::force_anim),
        );

        pa_widgets.insert(
            TypeId::of::<GravityAnimation>(),
            Box::new(EditorWidgets::gravity_anim),
        );

        pa_widgets.insert(
            TypeId::of::<StrayAnimation>(),
            Box::new(EditorWidgets::stray_anim),
        );

        let data = EditorData {
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_tab: Tab::EmitterSettings,
            texture_paths,
            selected_menu_idx: 0,
            selected_emitter_idx: 0,
            selected_new_par_anim: 0,
            selected_new_em_anim: 0,
            selected_new_post_fx: 0,
            selected_texture: 0,
            icon_textures,
            new_emitter_tag: String::from(""),
            profiling_results: Vec::new(),
            //display_event: None,
            //performance_event: None,
            emitter_settings: None,
        };

        Self { data, pa_widgets }
    }

    fn emitter_settings_tab(
        &mut self,
        state: &mut State,
        ui: &mut Ui,
        encoder: &mut CommandEncoder,
    ) {
        let data = &mut self.data;
        let emitter = &mut state.emitters[data.selected_emitter_idx];

        let mut emitter_settings = data
            .emitter_settings
            .get_or_insert_with(|| emitter.uniform.create_settings());

        if emitter.id() != &emitter_settings.id {
            emitter_settings = data
                .emitter_settings
                .insert(emitter.uniform.create_settings());
        }

        ui.add_space(5.0);

        Self::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.x, "Box yaw");
        Self::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.y, "Box pitch");
        Self::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.z, "Box roll");

        Self::create_degree_slider(ui, &mut emitter_settings.diff_width_deg, "Diffusion width");
        Self::create_degree_slider(ui, &mut emitter_settings.diff_depth_deg, "Diffusion depth");

        ui.add_space(5.0);
        create_label(ui, "Box dimensions (w, h, d)");

        ui.horizontal(|ui| {
            create_drag_value(ui, &mut emitter_settings.box_dimensions.x);
            create_drag_value(ui, &mut emitter_settings.box_dimensions.y);
            create_drag_value(ui, &mut emitter_settings.box_dimensions.z);
        });

        ui.add_space(5.0);
        create_label(ui, "Box position");

        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut emitter_settings.box_position.x).speed(0.1));
            ui.add(egui::DragValue::new(&mut emitter_settings.box_position.y).speed(0.1));
            ui.add(egui::DragValue::new(&mut emitter_settings.box_position.z).speed(0.1));
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            let col = &mut emitter_settings.particle_color;
            let mut particle_color = Rgba::from_rgba_unmultiplied(col.x, col.y, col.z, col.w);

            if color_edit_button_rgba(ui, &mut particle_color, Alpha::Opaque).changed() {
                col.x = particle_color.r();
                col.y = particle_color.g();
                col.z = particle_color.b();
                col.w = particle_color.a();
            };

            ui.label("Particle color");
            ui.add(
                Slider::new(&mut emitter_settings.hdr_mul, 1.0..=150.0).text("HDR multiplication"),
            );
        });

        ui.add_space(5.0);
        ui.add(
            Slider::new(&mut emitter_settings.particle_speed_min, 0.0..=50.0)
                .text("Particle emit speed min"),
        );
        ui.add(
            Slider::new(
                &mut emitter_settings.particle_speed_max,
                emitter_settings.particle_speed_min..=50.0,
            )
            .text("Particle emit speed max"),
        );
        ui.add_space(5.0);
        create_label(ui, "Spawn itemings");

        ui.add(
            egui::Slider::new(&mut emitter_settings.particle_lifetime_sec, 1.0..=40.0)
                .drag_value_speed(0.)
                .max_decimals(1)
                .step_by(0.1)
                .text("Particle lifetime (sec)"),
        );

        ui.add(
            egui::Slider::new(&mut emitter_settings.spawn_delay_sec, 0.1..=20.0)
                .drag_value_speed(0.)
                .max_decimals(1)
                .step_by(0.1)
                .text("Spawn delay (sec)"),
        );

        ui.add(egui::Slider::new(&mut emitter_settings.spawn_count, 1..=100).text("Spawn count"));

        ui.add_space(5.0);

        emitter_settings.recreate = ui.button("Update spawn settings").clicked();

        ui.add_space(5.0);

        create_label(ui, "Particle settings");

        ui.add_space(5.0);

        ui.add(
            Slider::new(&mut emitter_settings.particle_size_min, 0.01..=2.0)
                .text("Particle size min"),
        );
        ui.add(
            Slider::new(
                &mut emitter_settings.particle_size_max,
                emitter_settings.particle_size_min..=2.0,
            )
            .text("Particle size max"),
        );

        let combo_texture = ComboBox::from_label("Select texture").show_index(
            ui,
            &mut data.selected_texture,
            data.texture_paths.len(),
            |i| data.texture_paths[i].rich_text(),
        );

        if combo_texture.changed() {
            emitter.update_diffuse(&state.gfx, &mut data.texture_paths[data.selected_texture]);
        };

        self.update_emitter(state, encoder);
    }

    fn update_emitter(&mut self, state: &mut State, encoder: &mut wgpu::CommandEncoder) {
        // TODO move to APP and make events for recreating emitter!
        let data = &mut self.data;

        let State {
            camera,
            emitters,
            gfx,
            collection,
            ..
        } = state;

        let settings = data.emitter_settings.as_ref().unwrap();

        let (em, mut others) = emitters.split_item_mut(data.selected_emitter_idx);

        em.uniform.update_settings(settings);

        if settings.recreate {
            if em.is_light {
                *em = EmitterState::recreate_emitter(
                    RecreateEmitterOptions {
                        old_self: em,
                        gfx,
                        camera,
                        collection,
                        emitter_type: EmitterType::Lights,
                    },
                    encoder,
                );

                for other in others {
                    *other = EmitterState::recreate_emitter(
                        RecreateEmitterOptions {
                            old_self: other,
                            gfx,
                            camera,
                            collection,
                            emitter_type: EmitterType::Normal {
                                lights_layout: &em.bg_layout,
                            },
                        },
                        encoder,
                    );
                }
            } else {
                let lights = others.next().unwrap();

                *em = EmitterState::recreate_emitter(
                    RecreateEmitterOptions {
                        old_self: em,
                        gfx,
                        camera,
                        collection,
                        emitter_type: EmitterType::Normal {
                            lights_layout: &lights.bg_layout,
                        },
                    },
                    encoder,
                );
            }
        }
    }

    fn post_fx_tab(&mut self, state: &mut State, ui: &mut Ui, events: &mut Events) {
        let data = &mut self.data;
        let State {
            post_process,
            registered_post_fx,
            gfx,
            ..
        } = state;

        let effects = &mut post_process.effects;
        for _fx in effects.iter_mut() {
            // TODO
            //fx.create_ui(ui, gui);
            ui.separator();
        }

        ui.separator();

        ui.horizontal(|ui| {
            let sel_post_fx = &mut data.selected_new_post_fx;

            ComboBox::from_id_source("new-post-fx").show_index(
                ui,
                sel_post_fx,
                registered_post_fx.len(),
                |i| registered_post_fx[i].tag(),
            );

            if ui.button("Add post fx").clicked() {
                // TODO events!
                effects.push(registered_post_fx[*sel_post_fx].create_default(&FxOptions {
                    fx_state: &post_process.fx_state,
                    gfx,
                }));
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
                events.io_view = Some(ViewIOEvent::Idx(tex_output as u32))
            }
        });
    }

    pub fn create_title(ui: &mut Ui, str: &str) {
        ui.label(RichText::new(str).color(Color32::WHITE).size(16.0));
        ui.add_space(5.0);
    }

    pub fn create_degree_slider(ui: &mut Ui, val: &mut f32, str: &str) {
        ui.add(Slider::new(val, 0.0..=360.).text(str));
    }
}

impl EditorData {
    /// Creates list item header
    pub fn create_li_header(&self, ui: &mut Ui, title: &str) -> ListAction {
        let mut selected_action = ListAction::None;

        ui.horizontal_top(|ui| {
            Editor::create_title(ui, title);

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
