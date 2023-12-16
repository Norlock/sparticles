use super::{declarations::MenuCtx, MenuWidget};

pub struct NoneMenu;

impl MenuWidget for NoneMenu {
    fn title(&self) -> &'static str {
        "None"
    }

    fn draw_ui(&self, _menu_ctx: &mut MenuCtx) {}
}
