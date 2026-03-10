use hecs::Entity;

use crate::components::{EnemyKind, PickupKind, PowerupEffect};

// ---------------------------------------------------------------------------
// Audio IDs/commands — placed here so gameplay and audio are in one import.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SfxId {
    PlayerLaser,
    PlayerDied,
    PlayerPowerup,
    EnemyLaser,
    EnemyDestroyed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MusicId {
    Spaceshooter,
}

#[derive(Clone, Copy, Debug)]
pub enum MusicCommand {
    Play(MusicId),
    Stop,
    SetVolume(f32),
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
    PickupCollected { entity: Entity, kind: PickupKind },
    PowerupCollected { entity: Entity, effect: PowerupEffect },
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

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Drains and returns all pending events, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.queue)
    }
}
