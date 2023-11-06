pub type ID = String;

pub enum ViewIOEvent {
    Add,
    Subtract,
    Idx(u32),
}

#[derive(Default)]
pub struct Events {
    reset_camera: Option<bool>,
    create_emitter: Option<ID>,
    delete_emitter: Option<ID>,
    io_view: Option<ViewIOEvent>,
}

/// Every event is consumed when fetched
impl Events {
    pub fn set_reset_camera(&mut self) {
        self.reset_camera = Some(true);
    }

    pub fn reset_camera(&mut self) -> Option<bool> {
        self.reset_camera.take()
    }

    pub fn set_create_emitter(&mut self, id: ID) {
        self.create_emitter = Some(id);
    }

    /// Event will be removed when returned
    pub fn create_emitter(&mut self) -> Option<ID> {
        self.create_emitter.take()
    }

    pub fn set_delete_emitter(&mut self, id: ID) {
        self.delete_emitter = Some(id);
    }

    /// Event will be removed when returned
    pub fn delete_emitter(&mut self) -> Option<ID> {
        self.delete_emitter.take()
    }

    pub fn set_io_view(&mut self, event: ViewIOEvent) {
        self.io_view = Some(event);
    }

    pub fn get_io_view(&mut self) -> Option<ViewIOEvent> {
        self.io_view.take()
    }
}
