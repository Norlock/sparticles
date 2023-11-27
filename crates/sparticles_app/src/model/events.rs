use crate::util::ID;

pub enum ViewIOEvent {
    Add,
    Subtract,
    Idx(u32),
}

pub enum EventAction {
    Update,
}

#[derive(Default, PartialEq)]
pub enum GameState {
    #[default]
    Play,
    Pause,
}

/// Every option event is consumed when fetched to prevent repeating behaviour
#[derive(Default)]
pub struct Events {
    reset_camera: Option<EventAction>,
    create_emitter: Option<ID>,
    delete_emitter: Option<ID>,
    io_view: Option<ViewIOEvent>,
    game_state: GameState,
}

impl Events {
    pub fn set_reset_camera(&mut self) {
        self.reset_camera = Some(EventAction::Update);
    }

    pub fn reset_camera(&mut self) -> Option<EventAction> {
        self.reset_camera.take()
    }

    pub fn set_create_emitter(&mut self, id: ID) {
        self.create_emitter = Some(id);
    }

    pub fn create_emitter(&mut self) -> Option<ID> {
        self.create_emitter.take()
    }

    pub fn set_delete_emitter(&mut self, id: ID) {
        self.delete_emitter = Some(id);
    }

    pub fn delete_emitter(&mut self) -> Option<ID> {
        self.delete_emitter.take()
    }

    pub fn set_io_view(&mut self, event: ViewIOEvent) {
        self.io_view = Some(event);
    }

    pub fn get_io_view(&mut self) -> Option<ViewIOEvent> {
        self.io_view.take()
    }

    pub fn toggle_game_state(&mut self) {
        match &self.game_state {
            GameState::Play => self.game_state = GameState::Pause,
            GameState::Pause => self.game_state = GameState::Play,
        }
    }

    pub fn play(&self) -> bool {
        self.game_state == GameState::Play
    }
}
