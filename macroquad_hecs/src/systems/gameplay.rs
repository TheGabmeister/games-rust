use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::events::{GameEvent, SfxId, MusicId};
use crate::prefabs;
use crate::resources::Resources;

// ---------------------------------------------------------------------------
// Player movement
// ---------------------------------------------------------------------------

pub fn system_player_movement(world: &mut World, res: &Resources, dt: f32) {
    // query_mut yields Q::Item only — no entity in the tuple
    for (transform, _player) in world.query_mut::<(&mut Transform, &Player)>() {
        transform.pos += res.input.move_axis * PLAYER_SPEED * dt;
        transform.pos.x = transform.pos.x.clamp(20.0, SCREEN_WIDTH - 20.0);
        transform.pos.y = transform.pos.y.clamp(20.0, SCREEN_HEIGHT - 20.0);
    }
}

// ---------------------------------------------------------------------------
// Player firing
// ---------------------------------------------------------------------------

pub fn system_player_fire(world: &mut World, res: &mut Resources, dt: f32) {
    // Two-pass: collect fire info (drops query_mut borrow), then spawn.
    let mut fire_info: Option<(Vec2, f32)> = None;

    for (transform, weapon, _player) in world.query_mut::<(&Transform, &mut Weapon, &Player)>() {
        weapon.timer -= dt;
        if res.input.fire_held && weapon.timer <= 0.0 {
            fire_info = Some((transform.pos, weapon.bullet_speed));
            weapon.timer = weapon.cooldown;
        }
    }

    if let Some((pos, speed)) = fire_info {
        prefabs::spawn_player_bullet(world, pos - vec2(0.0, 20.0), speed);
        res.sfx_manager.play_sound(SfxId::PlayerLaser);
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

pub fn system_enemy_fire(world: &mut World, res: &mut Resources, dt: f32) {
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
        res.sfx_manager.play_sound(SfxId::EnemyLaser);
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

pub fn system_process_events(world: &mut World, res: &mut Resources) {
    const MAX_EVENTS_PER_TICK: usize = 1024;

    let mut events: VecDeque<GameEvent> = res.events.drain().into();
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_this_tick = false;
    let mut reset_requested = false;
    let mut processed_events = 0_usize;

    while let Some(event) = events.pop_front() {
        processed_events += 1;
        if processed_events > MAX_EVENTS_PER_TICK {
            #[cfg(debug_assertions)]
            eprintln!(
                "system_process_events: exceeded MAX_EVENTS_PER_TICK={MAX_EVENTS_PER_TICK}, dropping remaining events this tick"
            );
            break;
        }

        match event {
            GameEvent::BulletHitEnemy { bullet, enemy } => {
                to_despawn.insert(bullet);
                apply_damage_to_enemy(world, res, enemy, &mut to_despawn);
            }

            GameEvent::BulletHitPlayer { bullet } => {
                to_despawn.insert(bullet);
                apply_damage_to_player(world, res, &mut player_died_this_tick);
            }

            GameEvent::PlayerHit => {
                apply_damage_to_player(world, res, &mut player_died_this_tick);
            }

            GameEvent::EnemyDestroyed { .. } => {
                
            }

            GameEvent::PickupCollected { entity, kind } => {
                to_despawn.insert(entity);
                match kind {
                    PickupKind::Life => {
                        res.lives = (res.lives + 1).min(5);
                    }
                    PickupKind::Star => {
                        res.score += 500;
                    }
                }
                
            }

            GameEvent::PowerupCollected { entity, effect } => {
                to_despawn.insert(entity);
                // Template: extend with real powerup logic here.
                let _ = effect;
                
            }

            GameEvent::PlayerDied => {
                if res.lives > 0 {
                    res.lives -= 1;
                } else {
                    reset_requested = true;
                }
            }

            GameEvent::GameStarted => {
                res.music_manager.play_music(MusicId::Spaceshooter);
            }

            GameEvent::PlayerCaptured { boss: _ } => {}
            GameEvent::StageCleared => {}
        }

        if !res.events.is_empty() {
            events.extend(res.events.drain());
        }
    }

    if reset_requested {
        world.clear();
        prefabs::spawn_player(world, res);
        res.score = 0;
        to_despawn.clear();
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
    res: &mut Resources,
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

    res.sfx_manager.play_sound(SfxId::EnemyDestroyed);
    res.events.emit(GameEvent::EnemyDestroyed { entity: enemy, kind });
    res.score += score;
    to_despawn.insert(enemy);
}

fn apply_damage_to_player(
    world: &mut World,
    res: &mut Resources,
    player_died_this_tick: &mut bool,
) {
    if *player_died_this_tick {
        return;
    }

    // One-hit kill: if a player exists, this hit kills them.
    let player_exists = world.query::<&Player>().iter().next().is_some();
    if player_exists {
        res.events.emit(GameEvent::PlayerDied);
        res.sfx_manager.play_sound(SfxId::PlayerDied);
        *player_died_this_tick = true;
    }
}
