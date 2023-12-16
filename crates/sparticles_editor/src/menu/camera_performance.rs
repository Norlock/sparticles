use async_std::task;
use sparticles_app::{
    gui::egui::{self, Ui},
    model::{SparState, TonemapType},
    profiler::GpuTimerScopeResult,
};

use crate::Editor;

use super::{declarations::MenuCtx, MenuWidget};

pub struct CameraPerformanceMenu;

impl MenuWidget for CameraPerformanceMenu {
    fn title(&self) -> &'static str {
        "Camera & Performance"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        egui::Window::new("Camera & Performance")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .show(menu_ctx.ctx, |ui| {
                let SparState {
                    clock,
                    emitters,
                    gfx,
                    play,
                    camera,
                    ..
                } = menu_ctx.state;

                let data = &mut menu_ctx.emitter_data;
                let events = &mut menu_ctx.events;

                data.create_title(ui, "Emitter menu");
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
                        let total = display_performance(ui, &data.profiling_results);
                        Editor::create_label(
                            ui,
                            format!("{} - {:.3}μs", "Total GPU time", total * 1_000_000.),
                        );
                    });

                ui.separator();

                ui.add_space(5.0);

                ui.horizontal(|ui| {
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
            });
    }
}

fn display_performance(ui: &mut Ui, results: &[GpuTimerScopeResult]) -> f64 {
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
                    .show(ui, |ui| display_performance(ui, &scope.nested_scopes));
            });
        }
    }

    total_time
}
