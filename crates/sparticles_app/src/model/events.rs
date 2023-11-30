use crate::util::ID;

pub enum ViewIOEvent {
    Add,
    Subtract,
    Idx(u32),
}

pub enum EventAction {
    Update,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum GameState {
    #[default]
    Play,
    Pause,
}

/// Every option event is consumed when fetched to prevent repeating behaviour
#[derive(Default)]
pub struct Events {
    pub reset_camera: bool,
    pub create_emitter: Option<ID>,
    pub delete_emitter: Option<ID>,
    pub io_view: Option<ViewIOEvent>,
    pub game_state: GameState,
}

impl Events {
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
