use std::collections::VecDeque;

use hecs::Entity;
use hecs::World;

use crate::components::Enemy;
use crate::events::EventBus;
use crate::events::GameEvent;
use crate::managers::{GameDirector, MusicManager, SfxManager};
use crate::resources::{DespawnQueue, GameState};

fn stage_cleared(world: &World, despawns: &DespawnQueue) -> bool {
    world
        .query::<(Entity, &Enemy)>()
        .iter()
        .all(|(entity, _)| despawns.contains(entity))
}

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
    let mut stage_clear_queued = false;

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
                director.apply_powerup(world, player, effect, duration, sfx);
            }

            GameEvent::PlayerDied => {
                director.on_player_died(world, despawns, sfx);
            }

            GameEvent::GameStarted => {
                //music.play_music(crate::events::MusicId::Spaceshooter);
            }

            GameEvent::StageCleared => {
                director.on_stage_cleared();
            }
        }

        if !stage_clear_queued
            && director.state == GameState::Playing
            && stage_cleared(world, despawns)
        {
            events.push_back(GameEvent::StageCleared);
            stage_clear_queued = true;
        }
    }
}
