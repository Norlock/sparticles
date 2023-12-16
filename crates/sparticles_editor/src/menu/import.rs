use super::{declarations::MenuCtx, MenuWidget};
use async_std::{sync::RwLock, task};
use sparticles_app::{
    gui::egui::{self},
    loader::Model,
    model::GfxState,
};
use std::{collections::HashMap, sync::Arc};

pub struct ImportMenu;

impl MenuWidget for ImportMenu {
    fn title(&self) -> &'static str {
        "Import"
    }

    fn draw_ui(&self, menu_ctx: &mut MenuCtx) {
        let data = &mut menu_ctx.emitter_data;
        let collection = &menu_ctx.state.collection;
        let gfx = &menu_ctx.state.gfx;
        let mut import_file = None;

        egui::Window::new("Import settings")
            .vscroll(true)
            .default_height(800.)
            .title_bar(false)
            .default_rect(egui::Rect {
                min: [10., 10.].into(),
                max: egui::Pos2::default(),
            })
            .show(menu_ctx.ctx, |ui| {
                data.create_title(ui, "Import menu");

                let coll = task::block_on(collection.write());

                for file in data.model_files.iter() {
                    let filename = file.file_name().unwrap().to_str().unwrap();
                    let checked = &mut coll.contains_key(filename);

                    ui.separator();

                    ui.label(filename);

                    if ui.checkbox(checked, "Is imported").clicked() {
                        import_file = Some(filename.to_string());
                    }

                    ui.separator();
                }
            });

        if let Some(filename) = import_file {
            let gfx_clone = gfx.clone();
            let coll_clone = collection.clone();

            task::spawn(async move { load_import(gfx_clone, coll_clone, filename).await });
        }
    }
}

async fn load_import(
    gfx: Arc<RwLock<GfxState>>,
    collection: Arc<RwLock<HashMap<String, Model>>>,
    filename: String,
) {
    let model = Model::load_gltf(&gfx, &filename)
        .await
        .expect("Can't load model");

    let collection = &mut collection.write().await;

    collection.insert(filename.to_string(), model);
}
