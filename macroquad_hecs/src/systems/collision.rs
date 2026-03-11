use std::collections::HashSet;

use hecs::{Entity, World};
use macroquad::prelude::{vec2, Vec2};

use crate::components::{
    ActivePowerup, BoxCollider, Projectile, ProjectileOwner, CircleCollider, CollisionLayer, Enemy,
    EnemyKind, Pickup, PickupKind, Player, PowerupEffect, Transform,
};
use crate::events::{EventBus, GameEvent};

#[derive(Clone, Copy)]
enum ColliderShape {
    Box { half: Vec2 },
    Circle { radius: f32 },
}

#[derive(Clone, Copy)]
struct ColliderParticipant {
    entity: Entity,
    pos: Vec2,
    layer: CollisionLayer,
    shape: ColliderShape,
}

fn overlaps_aabb(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    let dx = (a_pos.x - b_pos.x).abs();
    let dy = (a_pos.y - b_pos.y).abs();
    dx <= a_half.x + b_half.x && dy <= a_half.y + b_half.y
}

fn overlaps_circle(a_pos: Vec2, a_radius: f32, b_pos: Vec2, b_radius: f32) -> bool {
    let r = a_radius + b_radius;
    (a_pos - b_pos).length_squared() <= r * r
}

fn overlaps_box_circle(box_pos: Vec2, box_half: Vec2, circle_pos: Vec2, circle_radius: f32) -> bool {
    let min = box_pos - box_half;
    let max = box_pos + box_half;
    let closest = vec2(
        circle_pos.x.clamp(min.x, max.x),
        circle_pos.y.clamp(min.y, max.y),
    );
    (circle_pos - closest).length_squared() <= circle_radius * circle_radius
}

fn overlaps_shapes(a: ColliderParticipant, b: ColliderParticipant) -> bool {
    match (a.shape, b.shape) {
        (ColliderShape::Box { half: a_half }, ColliderShape::Box { half: b_half }) => {
            overlaps_aabb(a.pos, a_half, b.pos, b_half)
        }
        (ColliderShape::Circle { radius: a_radius }, ColliderShape::Circle { radius: b_radius }) => {
            overlaps_circle(a.pos, a_radius, b.pos, b_radius)
        }
        (ColliderShape::Box { half }, ColliderShape::Circle { radius }) => {
            overlaps_box_circle(a.pos, half, b.pos, radius)
        }
        (ColliderShape::Circle { radius }, ColliderShape::Box { half }) => {
            overlaps_box_circle(b.pos, half, a.pos, radius)
        }
    }
}

fn layers_collide(a: CollisionLayer, b: CollisionLayer) -> bool {
    (a.mask & b.member) != 0 && (b.mask & a.member) != 0
}

fn canonical_pair(a: Entity, b: Entity) -> (Entity, Entity) {
    if a.to_bits().get() <= b.to_bits().get() {
        (a, b)
    } else {
        (b, a)
    }
}

fn is_player(world: &World, entity: Entity) -> bool {
    world.get::<&Player>(entity).is_ok()
}

fn has_enemy(world: &World, entity: Entity) -> bool {
    world.get::<&Enemy>(entity).is_ok()
}

fn enemy_kind(world: &World, entity: Entity) -> Option<EnemyKind> {
    world.get::<&Enemy>(entity).ok().map(|enemy| enemy.kind)
}

fn bullet_owner(world: &World, entity: Entity) -> Option<ProjectileOwner> {
    world.get::<&Projectile>(entity).ok().map(|bullet| bullet.owner)
}

fn pickup_kind(world: &World, entity: Entity) -> Option<PickupKind> {
    world.get::<&Pickup>(entity).ok().map(|pickup| pickup.kind)
}

fn powerup_effect(world: &World, entity: Entity) -> Option<PowerupEffect> {
    world
        .get::<&ActivePowerup>(entity)
        .ok()
        .map(|powerup| powerup.effect)
}

fn match_player_bullet_enemy(
    world: &World,
    a: Entity,
    b: Entity,
) -> Option<(Entity, Entity, EnemyKind)> {
    if bullet_owner(world, a) == Some(ProjectileOwner::Player) {
        if let Some(kind) = enemy_kind(world, b) {
            return Some((a, b, kind));
        }
    }

    if bullet_owner(world, b) == Some(ProjectileOwner::Player) {
        if let Some(kind) = enemy_kind(world, a) {
            return Some((b, a, kind));
        }
    }

    None
}

fn match_enemy_bullet_player(world: &World, a: Entity, b: Entity) -> Option<(Entity, Entity)> {
    if bullet_owner(world, a) == Some(ProjectileOwner::Enemy) && is_player(world, b) {
        return Some((a, b));
    }

    if bullet_owner(world, b) == Some(ProjectileOwner::Enemy) && is_player(world, a) {
        return Some((b, a));
    }

    None
}

fn match_player_enemy(world: &World, a: Entity, b: Entity) -> Option<Entity> {
    if is_player(world, a) && has_enemy(world, b) {
        return Some(a);
    }

    if is_player(world, b) && has_enemy(world, a) {
        return Some(b);
    }

    None
}

fn match_player_pickup(world: &World, a: Entity, b: Entity) -> Option<(Entity, PickupKind)> {
    if is_player(world, a) {
        if let Some(kind) = pickup_kind(world, b) {
            return Some((b, kind));
        }
    }

    if is_player(world, b) {
        if let Some(kind) = pickup_kind(world, a) {
            return Some((a, kind));
        }
    }

    None
}

fn match_player_powerup(
    world: &World,
    a: Entity,
    b: Entity,
) -> Option<(Entity, PowerupEffect)> {
    if is_player(world, a) {
        if let Some(effect) = powerup_effect(world, b) {
            return Some((b, effect));
        }
    }

    if is_player(world, b) {
        if let Some(effect) = powerup_effect(world, a) {
            return Some((a, effect));
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Main collision system
// ---------------------------------------------------------------------------

pub fn system_collision(world: &mut World, events: &mut EventBus) {
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    let mut player_died_emitted = false;

    // Snapshot all colliders first, then mutate world after overlap detection.
    let mut colliders: Vec<ColliderParticipant> = Vec::new();

    for (entity, transform, layer, collider) in world
        .query::<(Entity, &Transform, &CollisionLayer, &BoxCollider)>()
        .iter()
    {
        colliders.push(ColliderParticipant {
            entity,
            pos: transform.pos,
            layer: *layer,
            shape: ColliderShape::Box {
                half: collider.half,
            },
        });
    }

    for (entity, transform, layer, collider) in world
        .query::<(Entity, &Transform, &CollisionLayer, &CircleCollider)>()
        .iter()
    {
        colliders.push(ColliderParticipant {
            entity,
            pos: transform.pos,
            layer: *layer,
            shape: ColliderShape::Circle {
                radius: collider.radius,
            },
        });
    }

    let mut overlaps: Vec<(Entity, Entity)> = Vec::new();
    let mut seen_pairs: HashSet<(Entity, Entity)> = HashSet::new();

    for i in 0..colliders.len() {
        let a = colliders[i];
        for b in colliders.iter().copied().skip(i + 1) {
            if !layers_collide(a.layer, b.layer) {
                continue;
            }

            if !overlaps_shapes(a, b) {
                continue;
            }

            let pair = canonical_pair(a.entity, b.entity);
            if seen_pairs.insert(pair) {
                overlaps.push(pair);
            }
        }
    }

    for (a, b) in overlaps {
        if to_despawn.contains(&a) || to_despawn.contains(&b) {
            continue;
        }

        if let Some((bullet_entity, enemy_entity, kind)) = match_player_bullet_enemy(world, a, b) {
            to_despawn.insert(bullet_entity);
            to_despawn.insert(enemy_entity);
            events.emit(GameEvent::EnemyDestroyed {
                entity: enemy_entity,
                kind,
            });
            continue;
        }

        if let Some((bullet_entity, player_entity)) = match_enemy_bullet_player(world, a, b) {
            to_despawn.insert(bullet_entity);
            to_despawn.insert(player_entity);

            if !player_died_emitted {
                events.emit(GameEvent::PlayerDied);
                player_died_emitted = true;
            }
            continue;
        }

        if let Some(player_entity) = match_player_enemy(world, a, b) {
            to_despawn.insert(player_entity);

            if !player_died_emitted {
                events.emit(GameEvent::PlayerDied);
                player_died_emitted = true;
            }
            continue;
        }

        if let Some((pickup_entity, kind)) = match_player_pickup(world, a, b) {
            to_despawn.insert(pickup_entity);
            events.emit(GameEvent::PickupCollected {
                entity: pickup_entity,
                kind,
            });
            continue;
        }

        if let Some((powerup_entity, effect)) = match_player_powerup(world, a, b) {
            to_despawn.insert(powerup_entity);
            events.emit(GameEvent::PowerupCollected {
                entity: powerup_entity,
                effect,
            });
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}
