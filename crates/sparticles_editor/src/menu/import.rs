use sparticles_app::gui::egui;

use super::{menu::MenuCtx, MenuWidget};

pub struct ImportMenu;

impl MenuWidget for ImportMenu {
    fn title(&self) -> &'static str {
        "Import"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        let data = &mut menu_ctx.emitter_data;
        let collection = &menu_ctx.state.collection;

        egui::Window::new("General settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .show(menu_ctx.ctx, |ui| {
                data.create_title(ui, "Import menu");

                for file in data.model_files.iter() {
                    ui.group(|ui| {
                        let filename = file.file_name().unwrap().to_str().unwrap();
                        ui.label(filename);
                        ui.checkbox(&mut collection.contains_key(filename), "is imported");
                    });
                }
            });
    }
}
