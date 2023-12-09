use crate::{DynamicWidgets, EditorData};
use sparticles_app::{
    gui::egui,
    model::{SparEvents, SparState},
    wgpu::CommandEncoder,
};

pub struct MenuCtx<'a> {
    pub dyn_widgets: &'a mut DynamicWidgets,
    pub emitter_data: &'a mut EditorData,
    pub state: &'a mut SparState,
    pub events: &'a mut SparEvents,
    pub encoder: &'a mut CommandEncoder,
    pub ctx: &'a egui::Context,
}

pub trait MenuWidget {
    fn title(&self) -> &'static str;
    fn draw_ui(&self, menu_ctx: &mut MenuCtx);
}

impl PartialEq for dyn MenuWidget {
    fn eq(&self, other: &Self) -> bool {
        self.title() == other.title()
    }
}
