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
pub fn system_projectile_collision(
    world: &mut World,
    cmd: &mut CommandBuffer,
    res: &mut Resources,
) {
    // Snapshot projectiles once so we can then iterate enemies mutably.
    let all_projectiles: Vec<ProjectileSnapshot> = world
        .query::<(Entity, &Position, &Collider, &Projectile)>()
        .iter()
        .map(|(entity, pos, collider, projectile)| ProjectileSnapshot {
            entity,
            pos: pos.0,
            collider: *collider,
            projectile: *projectile,
        })
        .collect();

    let player_projectiles: Vec<ProjectileSnapshot> = all_projectiles
        .iter()
        .copied()
        .filter(|p| p.projectile.faction == Faction::Player)
        .collect();
    let spark_projectiles: Vec<ProjectileSnapshot> = all_projectiles
        .iter()
        .copied()
        .filter(|p| {
            p.projectile.faction == Faction::Enemy
                && p.projectile.kind == ProjectileKind::EnforcerSpark
        })
        .collect();

    if player_projectiles.is_empty() {
        return;
    }

    let mut consumed_projectiles: HashSet<Entity> = HashSet::new();
    let mut killed_enemies: HashSet<Entity> = HashSet::new();
    let mut hit_count: u32 = 0;
    let mut kill_score: u32 = 0;
    let mut spark_score: u32 = 0;

    // Sparks are destructible by player projectiles.
    for player_proj in &player_projectiles {
        if consumed_projectiles.contains(&player_proj.entity) {
            continue;
        }

        for spark_proj in &spark_projectiles {
            if consumed_projectiles.contains(&spark_proj.entity) {
                continue;
            }

            if overlaps(
                player_proj.collider,
                player_proj.pos,
                spark_proj.collider,
                spark_proj.pos,
            ) {
                consumed_projectiles.insert(player_proj.entity);
                consumed_projectiles.insert(spark_proj.entity);
                cmd.despawn(player_proj.entity);
                cmd.despawn(spark_proj.entity);
                spark_score += projectile_score(spark_proj.projectile.kind);
                hit_count += 1;
                break;
            }
        }
    }

    for (
        enemy_e,
        enemy_pos,
        enemy_col,
        enemy_vel,
        health,
        maybe_hit_slow,
        hit_reaction,
        invulnerable,
        enemy_kind,
    ) in &mut world.query::<(
        Entity,
        &Position,
        &Collider,
        &mut Velocity,
        &mut Health,
        Option<&mut HitSlow>,
        Option<&HitReaction>,
        Option<&Invulnerable>,
        &EnemyKind,
    )>() {
        if killed_enemies.contains(&enemy_e) {
            continue;
        }

        for proj in &player_projectiles {
            if consumed_projectiles.contains(&proj.entity) {
                continue;
            }

            if overlaps(proj.collider, proj.pos, *enemy_col, enemy_pos.0) {
                consumed_projectiles.insert(proj.entity);
                cmd.despawn(proj.entity);
                hit_count += 1;

                let is_invulnerable = invulnerable.is_some_and(|v| v.0);
                if is_invulnerable {
                    if let Some(hit_reaction) = hit_reaction {
                        if let Some(hit_slow) = maybe_hit_slow {
                            hit_slow.0 = hit_reaction.hit_slow_seconds.max(hit_slow.0);
                        }
                        let away = (enemy_pos.0 - proj.pos).normalize_or_zero();
                        enemy_vel.0 += away * hit_reaction.knockback_speed;
                    }
                    break; // projectile consumed on contact, no damage applied
                }

                let dmg = proj.projectile.damage.max(0);
                health.0 -= dmg;
                if health.0 <= 0 && killed_enemies.insert(enemy_e) {
                    cmd.despawn(enemy_e);
                    kill_score += enemy_score(*enemy_kind);
                    break; // dead enemy should not absorb more projectiles this frame
                }
            }
        }
    }

    // Credit score for kills plus spark interceptions.
    res.score += kill_score + spark_score;
    for _ in 0..hit_count {
        res.queue_sound(SoundId::Bump);
    }
}

/// Player death checks: contact-damage enemies or enemy projectiles.
pub fn system_player_contact_damage(world: &World) -> bool {
    let player = world
        .query::<With<(Entity, &Position, &Collider), &Player>>()
        .iter()
        .next()
        .map(|(_, pos, col)| (pos.0, *col));

    let Some((player_pos, player_col)) = player else {
        return false;
    };

    for (_, enemy_pos, enemy_col, contact_damage) in world
        .query::<(Entity, &Position, &Collider, &ContactDamage)>()
        .iter()
    {
        if contact_damage.damage > 0 && overlaps(player_col, player_pos, *enemy_col, enemy_pos.0) {
            return true;
        }
    }

    for (_, proj_pos, proj_col, projectile) in world
        .query::<(Entity, &Position, &Collider, &Projectile)>()
        .iter()
    {
        if projectile.faction == Faction::Enemy
            && projectile.damage > 0
            && overlaps(player_col, player_pos, *proj_col, proj_pos.0)
        {
            return true;
        }
    }

    false
}

fn enemy_score(kind: EnemyKind) -> u32 {
    match kind {
        EnemyKind::Sphereoid => 1000,
        EnemyKind::Enforcer => 150,
        _ => 1,
    }
}

fn projectile_score(kind: ProjectileKind) -> u32 {
    match kind {
        ProjectileKind::EnforcerSpark => 25,
        _ => 0,
    }
}
