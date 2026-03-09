use hecs::Entity;
use macroquad::prelude::*;

use crate::components::{EnemyKind, EnemyMode};

#[derive(Clone, Debug)]
pub enum GameEvent {
    SpawnPlayerProjectile {
        position: Vec2,
        barrel: u8,
    },
    SpawnEnemyProjectile {
        position: Vec2,
        velocity: Vec2,
    },
    EnemyDestroyed {
        entity: Entity,
        kind: EnemyKind,
        mode: EnemyMode,
        carrying_player: bool,
    },
    PlayerHit,
    PlayerCaptured {
        boss: Entity,
    },
    StageCleared,
    RescueCapturedShip,
    QueueBetrayedCapturedShip,
}
