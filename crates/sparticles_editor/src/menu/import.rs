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
                data.create_title(ui, "Import menu");

                //let data = &mut menu_ctx.data;
                //let SparState {
                //collection, camera, ..
                //} = menu_ctx.state;

                for file in data.model_files.iter() {
                    ui.group(|ui| {
                        ui.label(file.file_name().unwrap().to_str().unwrap());
                        ui.checkbox(&mut false, "is imported");
                    });
                }
            });
    }
}
