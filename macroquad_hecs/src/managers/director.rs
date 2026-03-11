use hecs::World;

use crate::components::{PickupKind, Player, Transform};
use crate::constants::PLAYER_START_LIVES;
use crate::constants::{PLAYER_START_X, PLAYER_START_Y};
use crate::constants::{PLAYER_MAX_LIVES, SCORE_PICKUP_STAR};
use crate::resources::GameState;

pub struct GameDirector {
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub state: GameState,
    pub debug_mode: bool,
}

impl Default for GameDirector {
    fn default() -> Self {
        Self {
            score: 0,
            lives: PLAYER_START_LIVES,
            high_score: 0,
            state: GameState::Playing,
            debug_mode: false,
        }
    }
}

impl GameDirector {
    pub fn reset_run(&mut self) {
        self.score = 0;
        self.lives = PLAYER_START_LIVES;
        self.state = GameState::Playing;
    }

    pub fn update_high_score(&mut self) {
        self.high_score = self.high_score.max(self.score);
    }

    pub fn add_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
    }

    pub fn add_lives_clamped(&mut self, amount: u32, max_lives: u32) {
        self.lives = self.lives.saturating_add(amount).min(max_lives);
    }

    pub fn on_player_died(&mut self, world: &mut World) {
        for (transform, _player) in world.query_mut::<(&mut Transform, &Player)>() {
            *transform = Transform::at(PLAYER_START_X, PLAYER_START_Y);
        }
    }

    pub fn apply_pickup_reward(&mut self, kind: PickupKind) {
        match kind {
            PickupKind::Life => self.add_lives_clamped(1, PLAYER_MAX_LIVES),
            PickupKind::Star => self.add_score(SCORE_PICKUP_STAR),
        }
    }
}
