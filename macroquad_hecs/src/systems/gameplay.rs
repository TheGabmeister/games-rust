use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::audio::SfxManager;
use crate::components::*;
use crate::constants::*;
use crate::events::{EventBus, GameEvent, MusicId, SfxId};
use crate::prefabs;
use crate::resources::{AudioState, GamePhase, GameState, InputState};

// ---------------------------------------------------------------------------
// Player movement
// ---------------------------------------------------------------------------

pub fn system_player_movement(world: &mut World, input: &InputState, dt: f32) {
    // query_mut yields Q::Item only — no entity in the tuple
    for (transform, _player) in world.query_mut::<(&mut Transform, &Player)>() {
        transform.pos += input.move_axis * PLAYER_SPEED * dt;
        transform.pos.x = transform.pos.x.clamp(20.0, SCREEN_WIDTH - 20.0);
        transform.pos.y = transform.pos.y.clamp(20.0, SCREEN_HEIGHT - 20.0);
    }
}

// ---------------------------------------------------------------------------
// Player firing
// ---------------------------------------------------------------------------

pub fn system_player_fire(world: &mut World, input: &InputState, sfx: &SfxManager, dt: f32) {
    // Two-pass: collect fire info (drops query_mut borrow), then spawn.
    let mut fire_info: Option<(Vec2, f32)> = None;

    for (transform, weapon, _player) in world.query_mut::<(&Transform, &mut Weapon, &Player)>() {
        weapon.timer -= dt;
        if input.fire_held && weapon.timer <= 0.0 {
            fire_info = Some((transform.pos, weapon.bullet_speed));
            weapon.timer = weapon.cooldown;
        }
    }

    if let Some((pos, speed)) = fire_info {
        prefabs::spawn_player_bullet(world, pos - vec2(0.0, 20.0), speed);
        sfx.play_sound(SfxId::PlayerLaser);
    }
}

// ---------------------------------------------------------------------------
// Enemy movement
// ---------------------------------------------------------------------------

pub fn system_enemy_movement(world: &mut World) {
    for (transform, _enemy) in world.query_mut::<(&mut Transform, &Enemy)>() {
        if transform.pos.y > SCREEN_HEIGHT + 32.0 {
            transform.pos.y = -32.0;
            transform.pos.x = macroquad::rand::gen_range(32.0, SCREEN_WIDTH - 32.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Enemy firing
// ---------------------------------------------------------------------------

pub fn system_enemy_fire(world: &mut World, sfx: &SfxManager, dt: f32) {
    // Two-pass: collect fire positions (drops query_mut borrow), then spawn.
    let mut fire_positions: Vec<(Vec2, f32)> = Vec::new();

    for (transform, weapon, _enemy) in world.query_mut::<(&Transform, &mut Weapon, &Enemy)>() {
        weapon.timer -= dt;
        if weapon.timer <= 0.0 {
            fire_positions.push((transform.pos, weapon.bullet_speed));
            weapon.timer = weapon.cooldown;
        }
    }

    for (pos, speed) in fire_positions {
        prefabs::spawn_enemy_bullet(world, pos + vec2(0.0, 20.0), speed);
        sfx.play_sound(SfxId::EnemyLaser);
    }
}

// ---------------------------------------------------------------------------
// Integrate velocities
// ---------------------------------------------------------------------------

pub fn system_integrate(world: &mut World, dt: f32) {
    for (transform, velocity) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.pos += velocity.linear * dt;
    }
}

// ---------------------------------------------------------------------------
// Lifetime tick + despawn
// ---------------------------------------------------------------------------

pub fn system_lifetime(world: &mut World, dt: f32) {
    // Include Entity in the type so .iter() yields (Entity, &mut Lifetime).
    // Block scope ensures QueryBorrow is dropped before world.despawn().
    let expired: Vec<Entity> = {
        let mut v = Vec::new();
        for (entity, lt) in world.query::<(Entity, &mut Lifetime)>().iter() {
            lt.remaining -= dt;
            if lt.remaining <= 0.0 {
                v.push(entity);
            }
        }
        v
    };

    for entity in expired {
        let _ = world.despawn(entity);
    }
}

// ---------------------------------------------------------------------------
// Cull off-screen bullets
// ---------------------------------------------------------------------------

pub fn system_cull_offscreen(world: &mut World) {
    const MARGIN: f32 = 64.0;

    let to_despawn: Vec<Entity> = {
        let mut v = Vec::new();
        // (Entity, &Transform, &Bullet) → yields flat 3-tuple from .iter()
        for (entity, transform, _bullet) in
            world.query::<(Entity, &Transform, &Bullet)>().iter()
        {
            let p = transform.pos;
            if p.x < -MARGIN
                || p.x > SCREEN_WIDTH + MARGIN
                || p.y < -MARGIN
                || p.y > SCREEN_HEIGHT + MARGIN
            {
                v.push(entity);
            }
        }
        v
    };

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

// ---------------------------------------------------------------------------
// Process events
// ---------------------------------------------------------------------------

pub fn system_process_events(
    world: &mut World,
    state: &mut GameState,
    events_bus: &mut EventBus,
    audio: &mut AudioState,
) {
    let mut events: VecDeque<GameEvent> = events_bus.drain().into();
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_this_tick = false;

    while let Some(event) = events.pop_front() {
        match event {
            GameEvent::EnemyHit { bullet, enemy } => {
                to_despawn.insert(bullet);
                apply_damage_to_enemy(
                    world,
                    state,
                    &mut events,
                    &audio.sfx,
                    enemy,
                    &mut to_despawn,
                );
            }

            GameEvent::PlayerHit { source } => {
                apply_damage_to_player(world, &mut events, &audio.sfx, &mut player_died_this_tick);
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
                audio.music.play_music(MusicId::Spaceshooter);
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
