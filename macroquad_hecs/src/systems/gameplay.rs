use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::events::{GameEvent, MusicId, SfxId};
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
        res.queue_sfx(SfxId::PlayerLaser);
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
        res.queue_sfx(SfxId::EnemyLaser);
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
    let events = res.events.drain();
    let mut to_despawn: Vec<Entity> = Vec::new();

    for event in events {
        match event {
            GameEvent::BulletHitEnemy { bullet, enemy } => {
                to_despawn.push(bullet);
                apply_damage_to_enemy(world, res, enemy, &mut to_despawn);
            }

            GameEvent::BulletHitPlayer { bullet } => {
                to_despawn.push(bullet);
                res.events.emit(GameEvent::PlayerHit);
            }

            GameEvent::PlayerHit => {
                apply_damage_to_player(world, res);
            }

            GameEvent::EnemyDestroyed { .. } => {
                res.queue_sfx(SfxId::EnemyDestroyed);
            }

            GameEvent::PickupTouched { pickup } => {
                // Read components; borrows from world.get() are dropped at end of block.
                let kind = world.get::<&Pickup>(pickup).ok().map(|p| p.kind);
                let powerup_effect =
                    world.get::<&ActivePowerup>(pickup).ok().map(|p| p.effect);

                if let Some(effect) = powerup_effect {
                    res.events
                        .emit(GameEvent::PowerupCollected { entity: pickup, effect });
                } else if let Some(kind) = kind {
                    res.events
                        .emit(GameEvent::PickupCollected { entity: pickup, kind });
                }
            }

            GameEvent::PickupCollected { entity, kind } => {
                to_despawn.push(entity);
                match kind {
                    PickupKind::Life => {
                        res.lives = (res.lives + 1).min(5);
                    }
                    PickupKind::Star => {
                        res.score += 500;
                    }
                }
                res.queue_sfx(SfxId::PlayerPowerup);
            }

            GameEvent::PowerupCollected { entity, effect } => {
                to_despawn.push(entity);
                // Template: extend with real powerup logic here.
                let _ = effect;
                res.queue_sfx(SfxId::PlayerPowerup);
            }

            GameEvent::PlayerDied => {
                if res.lives > 0 {
                    res.lives -= 1;
                } else {
                    world.clear();
                    prefabs::spawn_player(world, res);
                    res.score = 0;
                }
            }

            GameEvent::GameStarted => {
                res.music_manager.play_music(MusicId::Spaceshooter);
            }

            _ => {}
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
    res: &mut Resources,
    enemy: Entity,
    to_despawn: &mut Vec<Entity>,
) {
    let should_die = {
        if let Ok(mut hp) = world.get::<&mut Health>(enemy) {
            hp.current -= 1;
            hp.is_dead()
        } else {
            false
        }
        // hp RefMut is dropped here — world is free again
    };

    if should_die {
        // Read components before despawn; borrows auto-drop at end of let.
        let kind = world.get::<&Enemy>(enemy).ok().map(|e| e.kind);
        let score = world.get::<&ScoreValue>(enemy).ok().map(|s| s.0).unwrap_or(0);

        if let Some(kind) = kind {
            res.events.emit(GameEvent::EnemyDestroyed { entity: enemy, kind });
        }
        res.score += score;
        to_despawn.push(enemy);
    }
}

fn apply_damage_to_player(world: &mut World, res: &mut Resources) {
    // Include Entity in query type so .iter() yields (Entity, &Player, &Health).
    let player_entity: Option<Entity> = world
        .query::<(Entity, &Player, &Health)>()
        .iter()
        .next()
        .map(|(e, _p, _h)| e);

    if let Some(entity) = player_entity {
        let should_die = {
            if let Ok(mut hp) = world.get::<&mut Health>(entity) {
                hp.current -= 1;
                hp.is_dead()
            } else {
                false
            }
            // hp RefMut dropped here
        };

        if should_die {
            res.events.emit(GameEvent::PlayerDied);
        }

        res.queue_sfx(SfxId::PlayerDied);
    }
}
