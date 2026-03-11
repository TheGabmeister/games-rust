use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};

use crate::components::{Bullet, Enemy, PickupKind, ScoreValue};
use crate::constants::{PLAYER_MAX_LIVES, SCORE_PICKUP_STAR};
use crate::events::{EventBus, GameEvent, MusicId, SfxId};
use crate::managers::{GameDirector, MusicManager, SfxManager};
use crate::resources::GameState;

// ---------------------------------------------------------------------------
// Process events
// ---------------------------------------------------------------------------

pub fn system_process_events(
    world: &mut World,
    director: &mut GameDirector,
    events_bus: &mut EventBus,
    sfx: &mut SfxManager,
    music: &mut MusicManager,
) {
    let mut events: VecDeque<GameEvent> = events_bus.drain().into();
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_this_tick = false;

    while let Some(event) = events.pop_front() {
        match event {

            GameEvent::EnemyDestroyed { .. } => {}

            GameEvent::PickupCollected { entity, kind } => {
                to_despawn.insert(entity);
                apply_pickup_reward(director, kind);
            }

            GameEvent::PowerupCollected { entity, effect } => {
                to_despawn.insert(entity);
                // Template: extend with real powerup logic here.
                let _ = effect;
            }

            GameEvent::PlayerDied => {
                director.on_player_died();
            }

            GameEvent::GameStarted => {
                music.play_music(MusicId::Spaceshooter);
            }

            GameEvent::PlayerCaptured { boss: _ } => {}
            GameEvent::StageCleared => {
                if director.state == GameState::Playing {
                    director.state = GameState::Won;
                    director.update_high_score();
                } 
            }
        }
    }

    if director.state == GameState::Playing && !has_enemies(world) {
        events.push_back(GameEvent::StageCleared);
    }

    while let Some(event) = events.pop_front() {
        if let GameEvent::StageCleared = event {
            director.state = GameState::Won;
            director.update_high_score();
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn has_enemies(world: &World) -> bool {
    world.query::<&Enemy>().iter().next().is_some()
}

fn apply_pickup_reward(state: &mut GameDirector, kind: PickupKind) {
    match kind {
        PickupKind::Life => state.add_lives_clamped(1, PLAYER_MAX_LIVES),
        PickupKind::Star => state.add_score(SCORE_PICKUP_STAR),
    }
}
