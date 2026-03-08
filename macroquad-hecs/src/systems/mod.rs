use macroquad::prelude::*;
use hecs::*;
use ::rand::RngExt;

use crate::components::*;

// ── Constants ────────────────────────────────────────────────────────────────

const PLAYER_SPEED: f32 = 200.0;
const PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];

// ── Helpers ──────────────────────────────────────────────────────────────────

fn manhattan_dist(a: Vec2, b: Vec2) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as i32
}

// ── Spawning ─────────────────────────────────────────────────────────────────

pub fn batch_spawn_entities(world: &mut World, n: usize) {
    let mut rng = ::rand::rng();
    let to_spawn = (0..n).map(|_| {
        let pos  = Position(vec2(rng.random_range(0.0..800.0), rng.random_range(0.0..600.0)));
        let vel  = Velocity(Vec2::ZERO);
        let s    = Speed(rng.random_range(50.0..200.0));
        let hp   = Health(rng.random_range(30..50));
        let dmg  = Damage(rng.random_range(1..10));
        let kc   = KillCount(0);
        let tint = Tint(PALETTE[rng.random_range(0..PALETTE.len())]);
        (pos, vel, s, hp, dmg, kc, tint)
    });
    world.spawn_batch(to_spawn);
}

pub fn spawn_player(world: &mut World, texture: Texture2D) {
    world.spawn((
        Position(vec2(400.0, 300.0)),
        Velocity(Vec2::ZERO),
        Tint(WHITE),
        Player,
        texture,
    ));
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Write player velocity from keyboard input each frame.
/// The integrator system handles applying it to Position.
pub fn system_player_input(world: &mut World) {
    for (vel, _) in &mut world.query::<(&mut Velocity, &Player)>() {
        vel.0 = Vec2::ZERO;
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { vel.0.x += PLAYER_SPEED; }
        if is_key_down(KeyCode::Left)  || is_key_down(KeyCode::A) { vel.0.x -= PLAYER_SPEED; }
        if is_key_down(KeyCode::Down)  || is_key_down(KeyCode::S) { vel.0.y += PLAYER_SPEED; }
        if is_key_down(KeyCode::Up)    || is_key_down(KeyCode::W) { vel.0.y -= PLAYER_SPEED; }
    }
}

/// Set a random Velocity each frame for every NPC that has a Speed rating.
/// The Player has no Speed component, so it is automatically excluded.
pub fn system_wander_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Velocity, &Speed)>,
) {
    let mut rng = ::rand::rng();
    for (vel, speed) in query.query_mut(world) {
        vel.0 = vec2(
            rng.random_range(-speed.0..speed.0),
            rng.random_range(-speed.0..speed.0),
        );
    }
}

/// Apply Velocity to Position for all entities (NPCs and player alike).
pub fn system_integrate_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Position, &Velocity)>,
) {
    let dt = get_frame_time();
    for (pos, vel) in query.query_mut(world) {
        pos.0 += vel.0 * dt;
    }
}

/// Each entity with Health+Damage finds the nearest living neighbour within
/// ATTACK_RANGE and deals damage to it. O(n²) — fine for ≤ a few hundred entities.
pub fn system_fire_at_closest(world: &mut World) {
    for (id0, pos0, dmg0, kc0) in
        &mut world.query::<With<(Entity, &Position, &Damage, &mut KillCount), &Health>>()
    {
        // Skip if already killed this tick.
        if world.get::<&Health>(id0).map_or(true, |hp| hp.0 <= 0) { continue; }

        const ATTACK_RANGE: i32 = 80;

        let closest = world
            .query::<With<(Entity, &Position), &Health>>()
            .iter()
            .filter(|(id1, _)| *id1 != id0)
            .filter(|(_, pos1)| manhattan_dist(pos0.0, pos1.0) <= ATTACK_RANGE)
            .min_by_key(|(_, pos1)| manhattan_dist(pos0.0, pos1.0))
            .map(|(entity, _)| entity);

        let closest = match closest { Some(e) => e, None => continue };

        let mut hp1 = world.get::<&mut Health>(closest).unwrap();
        if hp1.0 > 0 {
            hp1.0 -= dmg0.0;
            if hp1.0 <= 0 { kc0.0 += 1; }
        }
    }
}

/// Despawn every entity whose Health has dropped to zero or below.
pub fn system_remove_dead(world: &mut World) {
    let mut to_remove: Vec<Entity> = Vec::new();
    for (id, hp) in &mut world.query::<(Entity, &Health)>() {
        if hp.0 <= 0 { to_remove.push(id); }
    }
    for entity in to_remove {
        world.despawn(entity).unwrap();
    }
}

/// Draw all entities: circles for NPCs, textured sprite for the player.
pub fn system_draw(world: &World) {
    for (pos, hp, kc, tint) in world.query::<(&Position, &Health, &KillCount, &Tint)>().iter() {
        draw_circle(pos.0.x, pos.0.y, 10.0, tint.0);
        draw_text(
            &format!("HP:{} K:{}", hp.0, kc.0),
            pos.0.x - 10.0,
            pos.0.y - 15.0,
            16.0,
            WHITE,
        );
    }
    for (pos, tex, _) in world.query::<(&Position, &Texture2D, &Player)>().iter() {
        let w = tex.width();
        let h = tex.height();
        draw_texture(tex, pos.0.x - w / 2.0, pos.0.y - h / 2.0, WHITE);
    }
}
