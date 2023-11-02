pub type Tag = String;

#[derive(Default)]
pub struct Events {
    reset_camera: Option<bool>,
    create_emitter: Option<Tag>,
    delete_emitter: Option<Tag>,
}

/// Every event is consumed when fetched
impl Events {
    pub fn set_reset_camera(&mut self) {
        self.reset_camera = Some(true);
    }

    /// Event will be removed when returned
    pub fn get_reset_camera(&mut self) -> Option<bool> {
        self.reset_camera.take()
    }

    pub fn set_create_emitter(&mut self, tag: Tag) {
        self.create_emitter = Some(tag);
    }

    /// Event will be removed when returned
    pub fn get_create_emitter(&mut self) -> Option<Tag> {
        self.create_emitter.take()
    }

    pub fn set_delete_emitter(&mut self, tag: Tag) {
        self.delete_emitter = Some(tag);
    }

    /// Event will be removed when returned
    pub fn get_delete_emitter(&mut self) -> Option<String> {
        self.delete_emitter.take()
    }
}
