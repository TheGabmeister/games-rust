use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};

use crate::audio::{MusicManager, SfxManager};
use crate::components::{Bullet, Enemy, PickupKind, Player, ScoreValue};
use crate::constants::{PLAYER_MAX_LIVES, SCORE_PICKUP_STAR};
use crate::events::{EventBus, GameEvent, MusicId, SfxId};
use crate::resources::{GamePhase, GameState};

// ---------------------------------------------------------------------------
// Process events
// ---------------------------------------------------------------------------

pub fn system_process_events(
    world: &mut World,
    state: &mut GameState,
    events_bus: &mut EventBus,
    sfx: &mut SfxManager,
    music: &mut MusicManager,
) {
    let mut events: VecDeque<GameEvent> = events_bus.drain().into();
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_this_tick = false;

    while let Some(event) = events.pop_front() {
        match event {
            GameEvent::EnemyHit { bullet, enemy } => {
                to_despawn.insert(bullet);
                apply_damage_to_enemy(world, state, &mut events, sfx, enemy, &mut to_despawn);
            }

            GameEvent::PlayerHit { source } => {
                apply_damage_to_player(world, &mut events, sfx, &mut player_died_this_tick);
                if world.get::<&Bullet>(source).is_ok() || world.get::<&Enemy>(source).is_ok() {
                    to_despawn.insert(source);
                }
            }

            GameEvent::EnemyDestroyed { .. } => {}

            GameEvent::PickupCollected { entity, kind } => {
                to_despawn.insert(entity);
                apply_pickup_reward(state, kind);
            }

            GameEvent::PowerupCollected { entity, effect } => {
                to_despawn.insert(entity);
                // Template: extend with real powerup logic here.
                let _ = effect;
            }

            GameEvent::PlayerDied => {
                if state.phase != GamePhase::Playing {
                    continue;
                }

                if state.lives > 1 {
                    state.lives -= 1;
                } else if state.lives == 1 {
                    state.lives = 0;
                    state.phase = GamePhase::Lost;
                    state.update_high_score();
                }
            }

            GameEvent::GameStarted => {
                music.play_music(MusicId::Spaceshooter);
            }

            GameEvent::PlayerCaptured { boss: _ } => {}
            GameEvent::StageCleared => {
                if state.phase == GamePhase::Playing {
                    state.phase = GamePhase::Won;
                    state.update_high_score();
                }
            }
        }
    }

    if state.phase == GamePhase::Playing && !has_enemies(world) {
        events.push_back(GameEvent::StageCleared);
    }

    while let Some(event) = events.pop_front() {
        if let GameEvent::StageCleared = event {
            state.phase = GamePhase::Won;
            state.update_high_score();
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn apply_damage_to_enemy(
    world: &mut World,
    state: &mut GameState,
    events: &mut VecDeque<GameEvent>,
    sfx: &SfxManager,
    enemy: Entity,
    to_despawn: &mut HashSet<Entity>,
) {
    // Ignore duplicate hit events for enemies already marked for despawn this tick.
    if to_despawn.contains(&enemy) {
        return;
    }

    // One-hit kill: any hit destroys entities that are actual Enemy actors.
    let kind = match world.get::<&Enemy>(enemy) {
        Ok(enemy_data) => enemy_data.kind,
        Err(_) => return,
    };
    let score = world.get::<&ScoreValue>(enemy).ok().map(|s| s.0).unwrap_or(0);

    sfx.play_sound(SfxId::EnemyDestroyed);
    events.push_back(GameEvent::EnemyDestroyed {
        entity: enemy,
        kind,
    });
    state.add_score(score);
    to_despawn.insert(enemy);
}

fn apply_damage_to_player(
    world: &mut World,
    events: &mut VecDeque<GameEvent>,
    sfx: &SfxManager,
    player_died_this_tick: &mut bool,
) {
    if *player_died_this_tick {
        return;
    }

    // One-hit kill: if a player exists, this hit kills them.
    let player_exists = world.query::<&Player>().iter().next().is_some();
    if player_exists {
        events.push_back(GameEvent::PlayerDied);
        sfx.play_sound(SfxId::PlayerDied);
        *player_died_this_tick = true;
    }
}

fn has_enemies(world: &World) -> bool {
    world.query::<&Enemy>().iter().next().is_some()
}

fn apply_pickup_reward(state: &mut GameState, kind: PickupKind) {
    match kind {
        PickupKind::Life => state.add_lives_clamped(1, PLAYER_MAX_LIVES),
        PickupKind::Star => state.add_score(SCORE_PICKUP_STAR),
    }
}
