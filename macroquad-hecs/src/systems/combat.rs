use macroquad::prelude::Vec2;
use hecs::*;

use crate::components::*;
use crate::collision::overlaps;
use crate::resources::{Resources, SoundId};

/// Collision: projectiles one-shot the first enemy they touch.
/// Both the projectile and the enemy are despawned immediately.
/// Two passes — collect then mutate — to satisfy the borrow checker.
pub fn system_projectile_collision(world: &mut World, res: &mut Resources) {
    // Pass 1: read all positions + colliders immutably.
    let projs: Vec<(Entity, Vec2, Collider)> = world
        .query::<With<(Entity, &Position, &Collider), &Projectile>>()
        .iter()
        .map(|(e, pos, col)| (e, pos.0, *col))
        .collect();

    let enemies: Vec<(Entity, Vec2, Collider)> = world
        .query::<With<(Entity, &Position, &Collider), &Enemy>>()
        .iter()
        .map(|(e, pos, col)| (e, pos.0, *col))
        .collect();

    // Collect (projectile, enemy) pairs — one hit per projectile.
    let mut hits: Vec<(Entity, Entity)> = Vec::new();
    for &(proj_e, proj_pos, proj_col) in &projs {
        for &(enemy_e, enemy_pos, enemy_col) in &enemies {
            if overlaps(proj_col, proj_pos, enemy_col, enemy_pos) {
                hits.push((proj_e, enemy_e));
                break; // one hit per projectile
            }
        }
    }

    // Pass 2: despawn both, credit score, queue hit sound.
    res.score += hits.len() as u32;
    for (proj_e, enemy_e) in hits {
        let _ = world.despawn(proj_e);
        let _ = world.despawn(enemy_e);
        res.queue_sound(SoundId::Bump);
    }
}
