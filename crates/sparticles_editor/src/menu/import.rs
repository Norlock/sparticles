use sparticles_app::gui::egui;

use super::{menu::MenuCtx, MenuWidget};

pub struct ImportMenu;

impl MenuWidget for ImportMenu {
    fn title(&self) -> &'static str {
        "Import"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        let data = &mut menu_ctx.emitter_data;

        egui::Window::new("General settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .show(menu_ctx.ctx, |ui| {
                ui.label("Select import");

                //let data = &mut menu_ctx.data;
                //let SparState {
                //collection, camera, ..
                //} = menu_ctx.state;

                egui::ComboBox::from_id_source("select-emitter").show_index(
                    ui,
                    &mut data.selected_emitter_idx,
                    data.model_files.len(),
                    |i| data.model_files[i].file_name().unwrap().to_str().unwrap(),
                );
            });
    }
}
