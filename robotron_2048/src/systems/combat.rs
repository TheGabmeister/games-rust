use macroquad::prelude::Vec2;
use hecs::*;

use crate::collision::overlaps;
use crate::components::*;
use crate::resources::{Resources, SoundId};

/// Collision: projectiles one-shot the first enemy they touch.
/// Both the projectile and enemy are queued for despawn via CommandBuffer.
pub fn system_projectile_collision(world: &World, cmd: &mut CommandBuffer, res: &mut Resources) {
    // Snapshot positions + colliders first.
    let projs: Vec<(Entity, Vec2, Collider, Projectile)> = world
        .query::<(Entity, &Position, &Collider, &Projectile)>()
        .iter()
        .map(|(e, pos, col, projectile)| (e, pos.0, *col, *projectile))
        .collect();

    let enemies: Vec<(Entity, Vec2, Collider)> = world
        .query::<With<(Entity, &Position, &Collider), &EnemyKind>>()
        .iter()
        .map(|(e, pos, col)| (e, pos.0, *col))
        .collect();

    // Queue one (projectile, enemy) hit per projectile.
    let mut hit_count: u32 = 0;
    for &(proj_e, proj_pos, proj_col, projectile) in &projs {
        if projectile.faction != Faction::Player {
            continue;
        }

        for &(enemy_e, enemy_pos, enemy_col) in &enemies {
            if overlaps(proj_col, proj_pos, enemy_col, enemy_pos) {
                cmd.despawn(proj_e);
                cmd.despawn(enemy_e);
                hit_count += 1;
                break; // one hit per projectile
            }
        }
    }

    // Credit score and queue one sound per hit.
    res.score += hit_count;
    for _ in 0..hit_count {
        res.queue_sound(SoundId::Bump);
    }
}
