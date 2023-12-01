use crate::util::ID;

#[derive(Debug)]
pub enum ViewIOEvent {
    Add,
    Subtract,
    Idx(u32),
}

pub enum EventAction {
    Update,
}

/// Every option event is consumed when fetched to prevent repeating behaviour
#[derive(Default, Debug)]
pub struct SparEvents {
    pub reset_camera: bool,
    pub create_emitter: Option<ID>,
    pub delete_emitter: Option<ID>,
    pub io_view: Option<ViewIOEvent>,
    pub toggle_play: bool,
}
