use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::events::SfxId;
use crate::managers::SfxManager;
use crate::prefabs;
use crate::resources::InputState;

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
    let mut fire_pos: Option<Vec2> = None;

    for (transform, fire_timer, _player) in
        world.query_mut::<(&Transform, &mut FireTimer, &Player)>()
    {
        fire_timer.timer -= dt;
        if input.fire_held && fire_timer.timer <= 0.0 {
            fire_pos = Some(transform.pos);
            fire_timer.timer = fire_timer.cooldown;
        }
    }

    if let Some(pos) = fire_pos {
        prefabs::spawn_player_bullet(world, pos - vec2(0.0, 20.0), PLAYER_BULLET_SPEED);
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
    let mut fire_positions: Vec<Vec2> = Vec::new();

    for (transform, fire_timer, _enemy) in world.query_mut::<(&Transform, &mut FireTimer, &Enemy)>()
    {
        fire_timer.timer -= dt;
        if fire_timer.timer <= 0.0 {
            fire_positions.push(transform.pos);
            fire_timer.timer = fire_timer.cooldown;
        }
    }

    for pos in fire_positions {
        prefabs::spawn_enemy_bullet(world, pos + vec2(0.0, 20.0), ENEMY_BULLET_SPEED);
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
        for (entity, transform, _bullet) in world.query::<(Entity, &Transform, &Bullet)>().iter() {
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
