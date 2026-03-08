use macroquad::prelude::Vec2;
use hecs::*;

use crate::components::*;
use crate::resources::{Resources, SoundId};

const HIT_RADIUS: f32 = 20.0; // pixels, projectile vs enemy

fn manhattan_dist(a: Vec2, b: Vec2) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as i32
}

/// Entities with Health+Damage find the nearest living neighbour in range and
/// attack it. O(n²) — acceptable up to a few hundred entities.
pub fn system_fire_at_closest(world: &mut World) {
    for (id0, pos0, dmg0, kc0) in
        &mut world.query::<With<(Entity, &Position, &Damage, &mut KillCount), &Health>>()
    {
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

/// Circle-vs-circle collision: projectiles damage the first enemy they touch,
/// then despawn. Two passes — collect then mutate — to satisfy the borrow checker.
pub fn system_projectile_collision(world: &mut World, res: &mut Resources) {
    // Pass 1: read all positions immutably, collect hits.
    // Each QueryBorrow is dropped at the end of its statement.
    let projs: Vec<(Entity, Vec2, i32)> = world
        .query::<With<(Entity, &Position, &Damage), &Projectile>>()
        .iter()
        .map(|(e, pos, dmg)| (e, pos.0, dmg.0))
        .collect();

    let enemies: Vec<(Entity, Vec2)> = world
        .query::<With<(Entity, &Position), &Enemy>>()
        .iter()
        .map(|(e, pos)| (e, pos.0))
        .collect();

    // Collect (projectile, enemy, damage) pairs — one hit per projectile.
    let mut hits: Vec<(Entity, Entity, i32)> = Vec::new();
    for &(proj_e, proj_pos, dmg) in &projs {
        for &(enemy_e, enemy_pos) in &enemies {
            if proj_pos.distance(enemy_pos) < HIT_RADIUS {
                hits.push((proj_e, enemy_e, dmg));
                break; // one hit per projectile
            }
        }
    }

    // Pass 2: apply damage and despawn the projectile.
    for (proj_e, enemy_e, dmg) in hits {
        if let Ok(mut hp) = world.get::<&mut Health>(enemy_e) {
            hp.0 -= dmg;
        }
        let _ = world.despawn(proj_e);
        res.queue_sound(SoundId::Bump);
    }
}
