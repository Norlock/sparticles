use egui_winit::egui;

pub trait DisplayWidget {
    fn display(&mut self) -> egui::Window;
}

pub struct Menu {
    title: String,
    display: Box<dyn DisplayWidget>,
}

impl PartialEq for Menu {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}
