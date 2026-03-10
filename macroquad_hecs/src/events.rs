use hecs::Entity;
use macroquad::prelude::*;

use crate::components::{EnemyKind};

#[derive(Clone, Debug)]
pub enum GameEvent {
    GameStarted,
    PlayerDied,
    EnemyDestroyed {
        entity: Entity,
        kind: EnemyKind,
    },
    PlayerHit,
    PlayerCaptured {
        boss: Entity,
    },
    StageCleared,
}

#[derive(Default)]
pub struct EventBus {
    queue: Vec<GameEvent>,
}

impl EventBus {
    pub fn emit(&mut self, event: GameEvent) { self.queue.push(event); }
    pub fn drain(&mut self) -> Vec<GameEvent> { std::mem::take(&mut self.queue) }
}