use std::collections::VecDeque;

use hecs::World;

use crate::components::{ActivePowerups, PowerupEffect};
use crate::events::EventBus;
use crate::events::GameEvent;
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
                director.on_enemy_destroyed(world, entity, sfx);
            }

            GameEvent::PickupCollected { entity: _, kind } => {
                director.apply_pickup_reward(kind);
            }

            GameEvent::PowerupCollected {
                entity: _,
                player,
                effect,
                duration,
            } => {
                if let Ok(mut powerups) = world.get::<&mut ActivePowerups>(player) {
                    match effect {
                        PowerupEffect::Bolt => {
                            powerups.bolt_remaining = powerups.bolt_remaining.max(duration);
                        }
                        PowerupEffect::Shield => {
                            powerups.shield_remaining = powerups.shield_remaining.max(duration);
                        }
                    }
                    sfx.play_sound(crate::events::SfxId::PlayerPowerup);
                }
            }

            GameEvent::PlayerDied => {
                director.on_player_died(world, despawns, sfx);
            }

            GameEvent::GameStarted => {
                //music.play_music(crate::events::MusicId::Spaceshooter);
            }

            GameEvent::StageCleared => {
                director.update_high_score();
            }
        }
    }
}
