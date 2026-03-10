use hecs::Entity;

use crate::components::{EnemyKind, PickupKind, PowerupEffect};

// ---------------------------------------------------------------------------
// Sound IDs — placed here so event handling and audio are in one import.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundId {
    PlayerLaser,
    PlayerDied,
    PlayerPowerup,
    EnemyLaser,
    EnemyDestroyed,
    MusicSpaceshooter,
}

// ---------------------------------------------------------------------------
// Game events
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum GameEvent {
    GameStarted,
    PlayerDied,
    PlayerHit,
    PlayerCaptured { boss: Entity },
    EnemyDestroyed { entity: Entity, kind: EnemyKind },
    /// Raw collision event — system_process_events looks up the Pickup component.
    PickupTouched { pickup: Entity },
    PickupCollected { entity: Entity, kind: PickupKind },
    PowerupCollected { entity: Entity, effect: PowerupEffect },
    BulletHitEnemy { bullet: Entity, enemy: Entity },
    BulletHitPlayer { bullet: Entity },
    StageCleared,
}

// ---------------------------------------------------------------------------
// Event bus
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct EventBus {
    queue: Vec<GameEvent>,
}

impl EventBus {
    pub fn emit(&mut self, event: GameEvent) {
        self.queue.push(event);
    }

    /// Drains and returns all pending events, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.queue)
    }
}
