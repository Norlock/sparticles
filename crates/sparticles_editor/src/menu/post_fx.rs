use async_std::task;
use sparticles_app::{
    fx::FxOptions,
    gui::egui,
    model::{events::ViewIOEvent, SparState},
};

use super::{declarations::MenuCtx, MenuWidget};

pub struct PostFxMenu;

impl MenuWidget for PostFxMenu {
    fn title(&self) -> &'static str {
        "Post FX"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        egui::Window::new("General settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .show(menu_ctx.ctx, |ui| {
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
                    ..
                } = state;

                let effects = &mut post_process.effects;
                for fx in effects.iter_mut() {
                    let type_id = (*fx.as_any()).type_id();

                    if let Some(widget) = widgets.fx_widgets.get_mut(&type_id) {
                        ui.group(|ui| widget(data, fx, ui));
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
            });
    }
}
