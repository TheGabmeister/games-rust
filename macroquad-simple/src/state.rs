#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

pub struct StateMachine {
    pub current: GameState,
}

impl StateMachine {
    pub fn new(initial: GameState) -> Self {
        Self { current: initial }
    }

    pub fn transition(&mut self, next: GameState) {
        self.current = next;
    }

    pub fn is(&self, state: &GameState) -> bool {
        &self.current == state
    }
}
