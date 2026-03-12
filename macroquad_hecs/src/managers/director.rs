use hecs::{Entity, World};

use crate::components::{PickupKind, Player, ScoreValue};
use crate::constants::PLAYER_START_LIVES;
use crate::constants::{PLAYER_MAX_LIVES, SCORE_PICKUP_STAR};
use crate::prefabs;
use crate::managers::SfxManager;
use crate::events::SfxId;
use crate::resources::{DespawnQueue, GameState};

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

    pub fn update_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
    }

    pub fn update_lives(&mut self, amount: i32) {
        if amount >= 0 {
            self.lives = self
                .lives
                .saturating_add(amount as u32)
                .min(PLAYER_MAX_LIVES);
        } else {
            self.lives = self.lives.saturating_sub(amount.unsigned_abs());
        }
    }

    pub fn on_enemy_destroyed(&mut self, world: &World, entity: Entity, sfx: &mut SfxManager) {
        sfx.play_sound((SfxId::EnemyDestroyed));
        if let Ok(score_value) = world.get::<&ScoreValue>(entity) {
            self.update_score(score_value.0);
        }
    }

    pub fn on_player_died(
        &mut self,
        world: &mut World,
        despawns: &mut DespawnQueue,
        sfx: &mut SfxManager,
    ) {
        let players: Vec<Entity> = world
            .query::<(Entity, &Player)>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();

        despawns.extend(players);
        sfx.play_sound((SfxId::PlayerDied));
        self.update_lives(-1);
        if self.lives == 0 {
            self.state = GameState::Lost;
        } else {
            prefabs::spawn_player(world);
        }
    }

    pub fn apply_pickup_reward(&mut self, kind: PickupKind) {
        match kind {
            PickupKind::Life => self.update_lives(1),
            PickupKind::Star => self.update_score(SCORE_PICKUP_STAR),
        }
    }
}
