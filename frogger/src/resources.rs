use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    TitleScreen,
    Playing,
    PlayerDead,    // death animation running; most systems paused
    LevelComplete, // brief celebration before resetting
    GameOver,
}

pub struct GameResources {
    pub phase: GamePhase,
    /// Velocity multiplier that scales every lane speed. Starts at 1.0, grows
    /// by SPEED_INCREMENT each level.
    pub speed_scale: f32,
}

impl GameResources {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::TitleScreen,
            speed_scale: 1.0,
        }
    }

    /// Advance speed for the next level (capped at MAX_SPEED_SCALE).
    pub fn advance_speed(&mut self) {
        self.speed_scale = (self.speed_scale + SPEED_INCREMENT).min(MAX_SPEED_SCALE);
    }
}
