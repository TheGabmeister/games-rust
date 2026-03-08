use hecs::*;
use macroquad::prelude::Vec2;
use std::collections::HashSet;

use crate::collision::overlaps;
use crate::components::*;
use crate::resources::{Resources, SoundId};

#[derive(Clone, Copy)]
struct ProjectileSnapshot {
    entity: Entity,
    pos: Vec2,
    collider: Collider,
    projectile: Projectile,
}

/// Resolve projectile-vs-enemy collisions.
/// Uses snapshot + deferred commands so we can safely mutate health and queue despawns.
pub fn system_projectile_collision(world: &mut World, cmd: &mut CommandBuffer, res: &mut Resources) {
    // Snapshot projectiles once so we can then iterate enemies mutably.
    let projectiles: Vec<ProjectileSnapshot> = world
        .query::<(Entity, &Position, &Collider, &Projectile)>()
        .iter()
        .map(|(entity, pos, collider, projectile)| ProjectileSnapshot {
            entity,
            pos: pos.0,
            collider: *collider,
            projectile: *projectile,
        })
        .filter(|p| p.projectile.faction == Faction::Player)
        .collect();

    if projectiles.is_empty() {
        return;
    }

    let mut consumed_projectiles: HashSet<Entity> = HashSet::new();
    let mut killed_enemies: HashSet<Entity> = HashSet::new();
    let mut hit_count: u32 = 0;
    let mut kill_count: u32 = 0;

    for (enemy_e, enemy_pos, enemy_col, health, invulnerable, _) in &mut world
        .query::<(
            Entity,
            &Position,
            &Collider,
            &mut Health,
            Option<&Invulnerable>,
            &EnemyKind,
        )>()
    {
        if killed_enemies.contains(&enemy_e) {
            continue;
        }

        for proj in &projectiles {
            if consumed_projectiles.contains(&proj.entity) {
                continue;
            }

            if overlaps(proj.collider, proj.pos, *enemy_col, enemy_pos.0) {
                consumed_projectiles.insert(proj.entity);
                cmd.despawn(proj.entity);
                hit_count += 1;

                let is_invulnerable = invulnerable.is_some_and(|v| v.0);
                if is_invulnerable {
                    break; // projectile consumed on contact, no damage applied
                }

                let dmg = proj.projectile.damage.max(0);
                health.0 -= dmg;
                if health.0 <= 0 && killed_enemies.insert(enemy_e) {
                    cmd.despawn(enemy_e);
                    kill_count += 1;
                    break; // dead enemy should not absorb more projectiles this frame
                }
            }
        }
    }

    // Credit score per kill; queue impact sound per registered hit.
    res.score += kill_count;
    for _ in 0..hit_count {
        res.queue_sound(SoundId::Bump);
    }
}

/// Grunt contact attack: touching the player is lethal for now.
pub fn system_grunt_contact_damage(world: &World) -> bool {
    let player = world
        .query::<With<(Entity, &Position, &Collider), &Player>>()
        .iter()
        .next()
        .map(|(_, pos, col)| (pos.0, *col));

    let Some((player_pos, player_col)) = player else {
        return false;
    };

    for (_, enemy_pos, enemy_col, kind) in world
        .query::<(Entity, &Position, &Collider, &EnemyKind)>()
        .iter()
    {
        if *kind == EnemyKind::Grunt && overlaps(player_col, player_pos, *enemy_col, enemy_pos.0) {
            return true;
        }
    }

    false
}
