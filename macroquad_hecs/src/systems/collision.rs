use std::collections::HashSet;

use hecs::{Entity, World};
use macroquad::prelude::Vec2;

use crate::components::{BoxCollider, Bullet, BulletOwner, Enemy, Player, Transform};
use crate::events::{EventBus, GameEvent};

fn overlaps_aabb(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    let dx = (a_pos.x - b_pos.x).abs();
    let dy = (a_pos.y - b_pos.y).abs();
    dx <= a_half.x + b_half.x && dy <= a_half.y + b_half.y
}

// ---------------------------------------------------------------------------
// Main collision system
// ---------------------------------------------------------------------------

pub fn system_collision(world: &mut World, events: &mut EventBus) {
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_emitted = false;

    // Snapshot all collider participants first, then apply mutations after detection.
    let enemies: Vec<(Entity, Transform, BoxCollider, Enemy)> = world
        .query::<(Entity, &Transform, &BoxCollider, &Enemy)>()
        .iter()
        .map(|(entity, transform, collider, enemy)| (entity, *transform, *collider, *enemy))
        .collect();

    let players: Vec<(Entity, Transform, BoxCollider)> = world
        .query::<(Entity, &Transform, &BoxCollider, &Player)>()
        .iter()
        .map(|(entity, transform, collider, _)| (entity, *transform, *collider))
        .collect();

    let player_bullets: Vec<(Entity, Transform, BoxCollider)> = world
        .query::<(Entity, &Transform, &BoxCollider, &Bullet)>()
        .iter()
        .filter_map(|(entity, transform, collider, bullet)| {
            (bullet.owner == BulletOwner::Player).then_some((entity, *transform, *collider))
        })
        .collect();

    let enemy_bullets: Vec<(Entity, Transform, BoxCollider)> = world
        .query::<(Entity, &Transform, &BoxCollider, &Bullet)>()
        .iter()
        .filter_map(|(entity, transform, collider, bullet)| {
            (bullet.owner == BulletOwner::Enemy).then_some((entity, *transform, *collider))
        })
        .collect();

    // Pass 1: player bullets vs enemies.
    for (bullet_entity, bullet_transform, bullet_collider) in player_bullets {
        if to_despawn.contains(&bullet_entity) {
            continue;
        }

        if let Some((enemy_entity, _, _, enemy)) = enemies
            .iter()
            .copied()
            .find(|(enemy_entity, enemy_transform, enemy_collider, _)| {
                !to_despawn.contains(enemy_entity)
                    && overlaps_aabb(
                        bullet_transform.pos,
                        bullet_collider.half,
                        enemy_transform.pos,
                        enemy_collider.half,
                    )
            })
        {
            to_despawn.insert(bullet_entity);
            to_despawn.insert(enemy_entity);
            events.emit(GameEvent::EnemyDestroyed {
                entity: enemy_entity,
                kind: enemy.kind,
            });
        }
    }

    // Pass 2: enemy bullets vs player ship.
    for (bullet_entity, bullet_transform, bullet_collider) in enemy_bullets {
        if to_despawn.contains(&bullet_entity) {
            continue;
        }

        if let Some((player_entity, _, _)) = players
            .iter()
            .copied()
            .find(|(player_entity, player_transform, player_collider)| {
                !to_despawn.contains(player_entity)
                    && overlaps_aabb(
                        bullet_transform.pos,
                        bullet_collider.half,
                        player_transform.pos,
                        player_collider.half,
                    )
            })
        {
            to_despawn.insert(bullet_entity);
            to_despawn.insert(player_entity);

            if !player_died_emitted {
                events.emit(GameEvent::PlayerDied);
                player_died_emitted = true;
            }
        }
    }

    // Pass 3: player ship vs enemy ships.
    for (player_entity, player_transform, player_collider) in players.iter().copied() {
        if to_despawn.contains(&player_entity) {
            continue;
        }

        let collided = enemies
            .iter()
            .any(|(enemy_entity, enemy_transform, enemy_collider, _)| {
                !to_despawn.contains(enemy_entity)
                    && overlaps_aabb(
                        player_transform.pos,
                        player_collider.half,
                        enemy_transform.pos,
                        enemy_collider.half,
                    )
            });

        if collided {
            to_despawn.insert(player_entity);

            if !player_died_emitted {
                events.emit(GameEvent::PlayerDied);
                player_died_emitted = true;
            }
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}