use std::collections::VecDeque;

use hecs::{Entity, World};

use crate::components::{Projectile, Enemy, PickupKind, ScoreValue};
use crate::events::{EventBus, GameEvent, MusicId, SfxId};
use crate::managers::{GameDirector, MusicManager, SfxManager};
use crate::resources::DespawnQueue;

// ---------------------------------------------------------------------------
// Process events
// ---------------------------------------------------------------------------

pub fn system_process_events(
    world: &mut World,
    director: &mut GameDirector,
    events_bus: &mut EventBus,
    despawns: &mut DespawnQueue,
    sfx: &mut SfxManager,
    music: &mut MusicManager,
) {
    let mut events: VecDeque<GameEvent> = events_bus.drain().into();

    while let Some(event) = events.pop_front() {
        match event {

            GameEvent::EnemyDestroyed { entity, kind: _ } => {
                director.on_enemy_destroyed();
            }

            GameEvent::PickupCollected { entity, kind } => {
                director.apply_pickup_reward(kind);
            }

            GameEvent::PowerupCollected { entity, effect } => {

            }

            GameEvent::PlayerDied => {
                director.on_player_died(world, despawns);
                sfx.play_sound((SfxId::PlayerDied));
            }

            GameEvent::GameStarted => {
                music.play_music(MusicId::Spaceshooter);
            }

            GameEvent::StageCleared => {
                director.update_high_score();
                
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn has_enemies(world: &World) -> bool {
    world.query::<&Enemy>().iter().next().is_some()
}
