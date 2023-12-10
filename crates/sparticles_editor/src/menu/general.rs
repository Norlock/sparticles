use crate::{DynamicWidgets, Editor, EditorData, IntoRichText};
use async_std::task;
use sparticles_app::{
    fx::{FxOptions, PostProcessState},
    gui::egui::{
        self,
        color_picker::{color_edit_button_rgba, Alpha},
        Rgba, Ui,
    },
    model::{
        camera::TonemapType, emitter_state::RecreateEmitterOptions, events::ViewIOEvent,
        EmitterState, EmitterType, SparState,
    },
    profiler::GpuTimerScopeResult,
    traits::Splitting,
    wgpu,
};

use super::{menu::MenuCtx, MenuWidget};

#[derive(PartialEq)]
pub enum Tab {
    EmitterSettings,
    PostFxSettings,
    ParticleAnimations,
    EmitterAnimations,
}

pub struct GeneralMenu;

impl MenuWidget for GeneralMenu {
    fn title(&self) -> &'static str {
        "General"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        egui::Window::new("General settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .show(menu_ctx.ctx, |ui| {
                let SparState {
                    clock,
                    emitters,
                    gfx,
                    post_process,
                    play,
                    camera,
                    ..
                } = menu_ctx.state;

                let data = &mut menu_ctx.emitter_data;
                let events = &mut menu_ctx.events;

                // Update gui info
                if clock.frame() % 20 == 0 && *play {
                    let gfx = &mut task::block_on(gfx.write());
                    let count: u64 = emitters.iter().map(|s| s.particle_count()).sum();

                    data.frame_time_text = clock.frame_time_text();
                    data.fps_text = clock.fps_text();
                    data.total_elapsed_text = clock.total_elapsed_text();
                    data.cpu_time_text = clock.cpu_time_text();
                    data.particle_count_text = format!("Particle count: {}", count);

                    if let Some(res) = gfx.process_frame() {
                        data.profiling_results = res;
                    }
                }

                Editor::create_label(ui, &data.fps_text);
                Editor::create_label(ui, &data.frame_time_text);
                Editor::create_label(ui, &data.cpu_time_text);
                Editor::create_label(ui, &data.total_elapsed_text);
                Editor::create_label(ui, &data.particle_count_text);
                ui.separator();

                egui::CollapsingHeader::new("Performance")
                    .id_source("total")
                    .show(ui, |ui| {
                        let total = self.display_performance(ui, &mut data.profiling_results);
                        Editor::create_label(
                            ui,
                            format!("{} - {:.3}μs", "Total GPU time", total * 1_000_000.),
                        );
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("select-emitter").show_index(
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

                    let emitter = &emitters[data.selected_emitter_idx];

                    if !emitter.is_light && ui.button("Remove emitter").clicked() {
                        let id = emitter.id().to_string();
                        events.delete_emitter = Some(id);
                        data.selected_emitter_idx = 0;
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
                        events.toggle_play = true;
                    }

                    egui::ComboBox::from_label("tonemapping")
                        .selected_text(camera.tonemap_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut camera.tonemap_type,
                                TonemapType::AcesNarkowicz,
                                TonemapType::AcesNarkowicz,
                            );
                            ui.selectable_value(
                                &mut camera.tonemap_type,
                                TonemapType::AcesHill,
                                TonemapType::AcesHill,
                            );
                            ui.selectable_value(
                                &mut camera.tonemap_type,
                                TonemapType::Uchimura,
                                TonemapType::Uchimura,
                            );
                            ui.selectable_value(
                                &mut camera.tonemap_type,
                                TonemapType::Lottes,
                                TonemapType::Lottes,
                            );
                        });
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
                    Tab::EmitterSettings => self.emitter_settings_tab(menu_ctx, ui),
                    Tab::PostFxSettings => self.post_fx_tab(menu_ctx, ui),
                    Tab::ParticleAnimations => self.particle_animations_tab(menu_ctx, ui),
                    Tab::EmitterAnimations => self.emitter_animations_tab(menu_ctx, ui),
                };
            });
    }
}

impl GeneralMenu {
    fn display_performance(&self, ui: &mut Ui, results: &[GpuTimerScopeResult]) -> f64 {
        let mut total_time = 0.;

        for scope in results.iter() {
            let time = scope.time.end - scope.time.start;
            total_time += time;
            let display_value = format!("{} - {:.3}μs", scope.label, time * 1_000_000.);

            Editor::create_label(ui, display_value);

            if !scope.nested_scopes.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(5.);
                    egui::CollapsingHeader::new("-- details --")
                        .id_source(&scope.label)
                        .show(ui, |ui| self.display_performance(ui, &scope.nested_scopes));
                });
            }
        }

        total_time
    }

    fn emitter_animations_tab(&self, menu_ctx: &mut MenuCtx, ui: &mut Ui) {
        let MenuCtx {
            dyn_widgets,
            emitter_data: data,
            state,
            ..
        } = menu_ctx;

        let emitter = &mut state.emitters[data.selected_emitter_idx];
        let registered_em_anims = &state.registry_em_anims;

        self.ui_emitter_animations(*dyn_widgets, *data, emitter, ui);

        ui.separator();

        ui.horizontal(|ui| {
            let sel_animation = &mut data.selected_new_em_anim;

            egui::ComboBox::from_id_source("new-particle-animation").show_index(
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

    fn particle_animations_tab(&self, menu_ctx: &mut MenuCtx, ui: &mut Ui) {
        let MenuCtx {
            dyn_widgets,
            emitter_data: data,
            state,
            ..
        } = menu_ctx;

        let SparState {
            emitters,
            registry_par_anims,
            ..
        } = state;

        let emitter = &mut emitters[data.selected_emitter_idx];
        self.ui_particle_animations(*dyn_widgets, *data, emitter, ui);

        ui.separator();

        ui.horizontal(|ui| {
            let sel_animation = &mut data.selected_new_par_anim;

            egui::ComboBox::from_id_source("new-particle-animation").show_index(
                ui,
                sel_animation,
                registry_par_anims.len(),
                |i| registry_par_anims[i].tag(),
            );

            if ui.button("Add animation").clicked() {
                let gfx = &task::block_on(state.gfx.read());
                emitter.push_particle_animation(
                    registry_par_anims[*sel_animation].create_default(gfx, emitter),
                );
            }
        });
    }

    fn ui_emitter_animations(
        &self,
        widgets: &mut DynamicWidgets,
        data: &mut EditorData,
        emitter: &mut EmitterState,
        ui: &mut Ui,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in emitter.emitter_animations.iter_mut() {
                    let type_id = (*anim.as_any()).type_id();

                    if let Some(widget) = widgets.em_widgets.get_mut(&type_id) {
                        ui.group(|ui| widget(data, anim, ui));
                    } else {
                        println!("widget not found");
                    }
                }
            });
    }

    fn ui_particle_animations(
        &self,
        widgets: &mut DynamicWidgets,
        data: &mut EditorData,
        emitter: &mut EmitterState,
        ui: &mut Ui,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in emitter.particle_animations.iter_mut() {
                    let type_id = (*anim.as_any()).type_id();

                    if let Some(widget) = widgets.pa_widgets.get_mut(&type_id) {
                        ui.group(|ui| widget(data, anim, ui));
                    } else {
                        println!("widget not found");
                    }
                }
            });
    }

    fn emitter_settings_tab(&self, menu_ctx: &mut MenuCtx, ui: &mut Ui) {
        let MenuCtx {
            emitter_data: data,
            state,
            encoder,
            ..
        } = menu_ctx;

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

        Editor::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.x, "Box yaw");
        Editor::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.y, "Box pitch");
        Editor::create_degree_slider(ui, &mut emitter_settings.box_rotation_deg.z, "Box roll");

        Editor::create_degree_slider(ui, &mut emitter_settings.diff_width_deg, "Diffusion width");
        Editor::create_degree_slider(ui, &mut emitter_settings.diff_depth_deg, "Diffusion depth");

        ui.add_space(5.0);
        Editor::create_label(ui, "Box dimensions (w, h, d)");

        ui.horizontal(|ui| {
            Editor::create_drag_value(ui, &mut emitter_settings.box_dimensions.x);
            Editor::create_drag_value(ui, &mut emitter_settings.box_dimensions.y);
            Editor::create_drag_value(ui, &mut emitter_settings.box_dimensions.z);
        });

        ui.add_space(5.0);
        Editor::create_label(ui, "Box position");

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
                egui::Slider::new(&mut emitter_settings.hdr_mul, 1.0..=150.0)
                    .text("HDR multiplication"),
            );
        });

        ui.add_space(5.0);
        ui.add(
            egui::Slider::new(&mut emitter_settings.particle_speed_min, 0.0..=50.0)
                .text("Particle emit speed min"),
        );
        ui.add(
            egui::Slider::new(
                &mut emitter_settings.particle_speed_max,
                emitter_settings.particle_speed_min..=50.0,
            )
            .text("Particle emit speed max"),
        );
        ui.add_space(5.0);
        Editor::create_label(ui, "Spawn itemings");

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

        Editor::create_label(ui, "Particle settings");

        ui.add_space(5.0);

        ui.add(
            egui::Slider::new(&mut emitter_settings.particle_size_min, 0.01..=2.0)
                .text("Particle size min"),
        );
        ui.add(
            egui::Slider::new(
                &mut emitter_settings.particle_size_max,
                emitter_settings.particle_size_min..=2.0,
            )
            .text("Particle size max"),
        );

        let combo_texture = egui::ComboBox::from_label("Select texture").show_index(
            ui,
            &mut data.selected_texture,
            data.texture_paths.len(),
            |i| data.texture_paths[i].rich_text(),
        );

        if combo_texture.changed() {
            let gfx = &task::block_on(state.gfx.read());
            emitter.update_diffuse(gfx, &mut data.texture_paths[data.selected_texture]);
        };

        task::block_on(self.update_emitter(*data, *state, *encoder));
    }

    async fn update_emitter(
        &self,
        data: &mut EditorData,
        state: &mut SparState,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // TODO move to APP and make events for recreating emitter!
        let SparState {
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
            //let gfx = &task::block_on(state.gfx.read());
            //let collection = &mut task::block_on(collection.write());

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
                )
                .await;

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
                    )
                    .await;
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
                )
                .await;
            }
        }
    }

    fn post_fx_tab(&self, menu_ctx: &mut MenuCtx, ui: &mut Ui) {
        let MenuCtx {
            dyn_widgets: widgets,
            emitter_data: data,
            state,
            events,
            ..
        } = menu_ctx;

        let SparState {
            post_process,
            registered_post_fx,
            gfx,
            ..
        } = state;

        let effects = &mut post_process.effects;
        for fx in effects.iter_mut() {
            let type_id = (*fx.as_any()).type_id();

            if let Some(widget) = widgets.fx_widgets.get_mut(&type_id) {
                ui.group(|ui| widget(*data, fx, ui));
            } else {
                println!("widget not found");
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            let sel_post_fx = &mut data.selected_new_post_fx;

            egui::ComboBox::from_id_source("new-post-fx").show_index(
                ui,
                sel_post_fx,
                registered_post_fx.len(),
                |i| registered_post_fx[i].tag(),
            );

            if ui.button("Add post fx").clicked() {
                let gfx = &task::block_on(state.gfx.read());

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
}
