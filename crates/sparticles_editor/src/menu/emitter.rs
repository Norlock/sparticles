use crate::{DynamicWidgets, Editor, EditorData};
use async_std::task;
use sparticles_app::{
    fx::PostProcessState,
    gui::egui::{
        self,
        color_picker::{color_edit_button_rgba, Alpha},
        scroll_area::ScrollBarVisibility,
        Color32, Rgba, RichText, Ui,
    },
    model::{emitter_state::RecreateEmitterOptions, EmitterState, EmitterType, SparState},
    traits::Splitting,
    wgpu,
};

use super::{declarations::MenuCtx, MenuWidget};

#[derive(PartialEq)]
pub enum Tab {
    EmitterSettings,
    ModelSettings,
    ParticleAnimations,
    EmitterAnimations,
}

pub struct EmitterMenu;

impl MenuWidget for EmitterMenu {
    fn title(&self) -> &'static str {
        "Emitter"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        egui::Window::new("Emitter settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .default_pos([10., 10.])
            .show(menu_ctx.ctx, |ui| {
                let SparState {
                    emitters,
                    post_process,
                    ..
                } = menu_ctx.state;

                let data = &mut menu_ctx.emitter_data;
                let events = &mut menu_ctx.events;

                data.create_title(ui, "Emitter menu");

                ui.horizontal(|ui| {
                    ui.label("New emitter tag:");
                    ui.add(
                        egui::TextEdit::singleline(&mut data.new_emitter_tag).desired_width(100.),
                    );

                    ui.add_space(4.0);

                    let is_enabled = 3 <= data.new_emitter_tag.len()
                        && emitters.iter().all(|em| em.id() != data.new_emitter_tag);

                    if ui
                        .add_enabled(is_enabled, egui::Button::new("Add emitter"))
                        .clicked()
                    {
                        events.create_emitter = Some(data.new_emitter_tag.to_string());
                        data.new_emitter_tag = "".to_string();
                    }
                });

                ui.add_space(6.0);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Export settings").clicked() {
                        EmitterState::export(emitters);
                        PostProcessState::export(post_process);
                    }

                    ui.add_space(4.0);

                    egui::ComboBox::from_id_source("select-emitter").show_index(
                        ui,
                        &mut data.selected_emitter_idx,
                        emitters.len(),
                        |i| emitters[i].id(),
                    );

                    ui.add_space(4.0);

                    let emitter = &emitters[data.selected_emitter_idx];
                    if !emitter.is_light && ui.button("Remove emitter").clicked() {
                        let id = emitter.id().to_string();
                        events.delete_emitter = Some(id);
                        data.selected_emitter_idx = 0;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut data.selected_tab,
                        Tab::EmitterSettings,
                        "Spawn settings",
                    );
                    ui.selectable_value(
                        &mut data.selected_tab,
                        Tab::ModelSettings,
                        "Model settings",
                    );
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
                    Tab::EmitterSettings => task::block_on(self.emitter_settings_tab(menu_ctx, ui)),
                    Tab::ModelSettings => task::block_on(model_settings(menu_ctx, ui)),
                    Tab::ParticleAnimations => self.particle_animations_tab(menu_ctx, ui),
                    Tab::EmitterAnimations => self.emitter_animations_tab(menu_ctx, ui),
                };
            });
    }
}

impl EmitterMenu {
    fn emitter_animations_tab(&self, menu_ctx: &mut MenuCtx, ui: &mut Ui) {
        let MenuCtx {
            dyn_widgets,
            emitter_data: data,
            state,
            ..
        } = menu_ctx;

        let emitter = &mut state.emitters[data.selected_emitter_idx];
        let registered_em_anims = &state.registry_em_anims;

        ui_emitter_animations(dyn_widgets, data, emitter, ui);

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
        ui_particle_animations(dyn_widgets, data, emitter, ui);

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

    async fn emitter_settings_tab(&self, menu_ctx: &mut MenuCtx<'_>, ui: &mut Ui) {
        let MenuCtx {
            emitter_data: data,
            state,
            encoder,
            ..
        } = menu_ctx;

        let uniform = &mut state.emitters[data.selected_emitter_idx].uniform;
        data.sync_emitter_settings(&uniform);
        let emitter_settings = data.emitter_settings.as_mut().unwrap();

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

        uniform.update_settings(&emitter_settings);

        if emitter_settings.recreate {
            recreate_emitter(data, state, encoder).await;
        }
    }
}

async fn model_settings(menu_ctx: &mut MenuCtx<'_>, ui: &mut Ui) {
    let MenuCtx {
        emitter_data: data,
        state,
        ..
    } = menu_ctx;

    let uniform = &mut state.emitters[data.selected_emitter_idx].uniform;
    data.sync_emitter_settings(&uniform);
    let emitter_settings = data.emitter_settings.as_mut().unwrap();

    let mesh = &mut emitter_settings.mesh;
    let mat = &mut emitter_settings.material;
    let collection = state.collection.read().await;

    fn custom_header(ui: &mut Ui, title: &str) {
        ui.add_space(10.0);

        ui.label(RichText::new(title).size(14.).color(Color32::WHITE));

        ui.add_space(10.0);
    }

    fn horizontal_scroll<R>(ui: &mut Ui, id_src: &str, add_contents: impl FnOnce(&mut Ui) -> R) {
        egui::ScrollArea::vertical()
            .id_source(id_src)
            .auto_shrink([true; 2])
            .hscroll(true)
            .max_height(150.)
            .max_width(f32::INFINITY)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| add_contents(ui));
            });
    }

    custom_header(ui, "Mesh colllection");

    horizontal_scroll(ui, "mesh_coll", |ui| {
        for (key, _) in collection.iter() {
            if ui
                .selectable_label(mesh.collection_id == *key, key)
                .clicked()
            {
                mesh.collection_id = key.clone();

                let mesh_model = collection.get(&mesh.collection_id).unwrap();
                mesh.mesh_id = mesh_model.meshes.keys().next().unwrap().to_string();
            }
        }
    });

    let mesh_model = collection.get(&mesh.collection_id).unwrap();

    custom_header(ui, "Mesh");

    horizontal_scroll(ui, "mesh", |ui| {
        for (key, _) in mesh_model.meshes.iter() {
            if ui.selectable_label(mesh.mesh_id == *key, key).clicked() {
                mesh.mesh_id = key.clone();
            }
        }
    });

    // TODO FIX
    //custom_header(ui, "Material collection");

    //horizontal_scroll(ui, "mat_coll", |ui| {
    //for (key, _) in collection.iter() {
    //if ui
    //.selectable_label(mat.collection_id == *key, key)
    //.clicked()
    //{
    //mat.collection_id = key.clone();

    //let mat_model = collection.get(&mat.collection_id).unwrap();
    //mat.material_id = mat_model.materials.keys().next().unwrap().to_string();
    //}
    //}
    //});

    //let mat_model = collection.get(&mat.collection_id).unwrap();

    //custom_header(ui, "Material");

    //horizontal_scroll(ui, "mat", |ui| {
    //for (key, _) in mat_model.materials.iter() {
    //if ui.selectable_label(mat.material_id == *key, key).clicked() {
    //mat.material_id = key.clone();
    //}
    //}
    //});

    ui.add_space(10.);

    uniform.update_settings(&emitter_settings);
}

fn ui_emitter_animations(
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

async fn recreate_emitter(
    data: &mut EditorData,
    state: &mut SparState,
    encoder: &mut wgpu::CommandEncoder,
) {
    let SparState {
        camera,
        emitters,
        gfx,
        collection,
        ..
    } = state;

    let (em, mut others) = emitters.split_item_mut(data.selected_emitter_idx);

    if em.is_light {
        *em = EmitterState::recreate_emitter(
            RecreateEmitterOptions {
                old_self: em,
                gfx,
                camera,
                collection,
                emitter_type: EmitterType::Lights,
                terrain_bg_layout: &state.terrain_generator.env_bg_layout,
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
                    terrain_bg_layout: &state.terrain_generator.env_bg_layout,
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
                terrain_bg_layout: &state.terrain_generator.env_bg_layout,
            },
            encoder,
        )
        .await;
    }
}
