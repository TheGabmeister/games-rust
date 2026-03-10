use std::cell::RefCell;

use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    ActivePowerup, BoxCollider, CircleCollider, CollisionLayer, Enemy, Pickup, Transform,
};
use crate::constants::*;
use crate::events::{EventBus, GameEvent};

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

fn aabb_overlaps(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    let dx = (a_pos.x - b_pos.x).abs();
    let dy = (a_pos.y - b_pos.y).abs();
    dx < (a_half.x + b_half.x) && dy < (a_half.y + b_half.y)
}

fn circles_overlap(a_pos: Vec2, a_r: f32, b_pos: Vec2, b_r: f32) -> bool {
    let r_sum = a_r + b_r;
    (a_pos - b_pos).length_squared() < r_sum * r_sum
}

fn circle_aabb_overlaps(c_pos: Vec2, r: f32, b_pos: Vec2, b_half: Vec2) -> bool {
    let nearest = vec2(
        c_pos.x.clamp(b_pos.x - b_half.x, b_pos.x + b_half.x),
        c_pos.y.clamp(b_pos.y - b_half.y, b_pos.y + b_half.y),
    );
    (c_pos - nearest).length_squared() < r * r
}

// ---------------------------------------------------------------------------
// Snapshot structs (released before event emission)
// ---------------------------------------------------------------------------

struct AabbEntry {
    entity: Entity,
    pos: Vec2,
    half: Vec2,
    layer: CollisionLayer,
}

struct CircleEntry {
    entity: Entity,
    pos: Vec2,
    radius: f32,
    layer: CollisionLayer,
}

// ---------------------------------------------------------------------------
// Per-frame scratch buffers (reused across calls to avoid allocation)
// ---------------------------------------------------------------------------

thread_local! {
    static SCRATCH: RefCell<(Vec<AabbEntry>, Vec<CircleEntry>)> =
        RefCell::new((Vec::new(), Vec::new()));
}

// ---------------------------------------------------------------------------
// Main collision system
// ---------------------------------------------------------------------------

/// Tests all collidable entity pairs and emits events via the EventBus.
/// Uses two-pass (snapshot → test) to avoid borrow conflicts with hecs.
pub fn system_collision(world: &mut World, events: &mut EventBus) {
    SCRATCH.with(|s| {
        let (aabbs, circles) = &mut *s.borrow_mut();

        // Pass 1: snapshot collidable entities (releases QueryBorrow)
        aabbs.clear();
        aabbs.extend(
            world
                .query::<(Entity, &Transform, &BoxCollider, &CollisionLayer)>()
                .iter()
                .map(|(entity, t, c, l)| AabbEntry {
                    entity,
                    pos: t.pos,
                    half: c.half,
                    layer: *l,
                }),
        );

        circles.clear();
        circles.extend(
            world
                .query::<(Entity, &Transform, &CircleCollider, &CollisionLayer)>()
                .iter()
                .map(|(entity, t, c, l)| CircleEntry {
                    entity,
                    pos: t.pos,
                    radius: c.radius,
                    layer: *l,
                }),
        );

        // Pass 2: test pairs and emit events
        // AABB vs AABB
        for i in 0..aabbs.len() {
            for j in (i + 1)..aabbs.len() {
                let a = &aabbs[i];
                let b = &aabbs[j];
                if layers_interact(a.layer, b.layer) && aabb_overlaps(a.pos, a.half, b.pos, b.half)
                {
                    emit_event(a.entity, a.layer, b.entity, b.layer, world, events);
                }
            }
        }

        // Circle vs Circle
        for i in 0..circles.len() {
            for j in (i + 1)..circles.len() {
                let a = &circles[i];
                let b = &circles[j];
                if layers_interact(a.layer, b.layer)
                    && circles_overlap(a.pos, a.radius, b.pos, b.radius)
                {
                    emit_event(a.entity, a.layer, b.entity, b.layer, world, events);
                }
            }
        }

        // Circle vs AABB
        for c in circles.iter() {
            for a in aabbs.iter() {
                if layers_interact(c.layer, a.layer)
                    && circle_aabb_overlaps(c.pos, c.radius, a.pos, a.half)
                {
                    emit_event(c.entity, c.layer, a.entity, a.layer, world, events);
                }
            }
        }
    });
}

fn layers_interact(a: CollisionLayer, b: CollisionLayer) -> bool {
    (a.mask & b.member) != 0 || (b.mask & a.member) != 0
}

fn emit_event(
    ea: Entity,
    la: CollisionLayer,
    eb: Entity,
    lb: CollisionLayer,
    world: &World,
    events: &mut EventBus,
) {
    // Identify roles by layer membership
    let a_is_player_bullet = (la.member & LAYER_PLAYER_BULLET) != 0;
    let b_is_player_bullet = (lb.member & LAYER_PLAYER_BULLET) != 0;
    let a_is_enemy_bullet = (la.member & LAYER_ENEMY_BULLET) != 0;
    let b_is_enemy_bullet = (lb.member & LAYER_ENEMY_BULLET) != 0;
    let a_is_player = (la.member & LAYER_PLAYER) != 0;
    let b_is_player = (lb.member & LAYER_PLAYER) != 0;
    let a_is_enemy = (la.member & LAYER_ENEMY) != 0;
    let b_is_enemy = (lb.member & LAYER_ENEMY) != 0;
    let a_is_pickup = (la.member & LAYER_PICKUP) != 0;
    let b_is_pickup = (lb.member & LAYER_PICKUP) != 0;

    if a_is_player_bullet && b_is_enemy {
        if let Ok(enemy) = world.get::<&Enemy>(eb) {
            events.emit(GameEvent::EnemyDestroyed {
                entity: eb,
                kind: enemy.kind,
            });
        }
    } else if b_is_player_bullet && a_is_enemy {
        if let Ok(enemy) = world.get::<&Enemy>(ea) {
            events.emit(GameEvent::EnemyDestroyed {
                entity: ea,
                kind: enemy.kind,
            });
        }
    } else if a_is_enemy_bullet && b_is_player {
        events.emit(GameEvent::PlayerHit { source: ea });
    } else if b_is_enemy_bullet && a_is_player {
        events.emit(GameEvent::PlayerHit { source: eb });
    } else if a_is_enemy && b_is_player {
        events.emit(GameEvent::PlayerHit { source: ea });
    } else if b_is_enemy && a_is_player {
        events.emit(GameEvent::PlayerHit { source: eb });
    } else if a_is_pickup {
        emit_pickup_event(world, events, ea);
    } else if b_is_pickup {
        emit_pickup_event(world, events, eb);
    }
}

fn emit_pickup_event(world: &World, events: &mut EventBus, entity: Entity) {
    if let Some(effect) = world.get::<&ActivePowerup>(entity).ok().map(|p| p.effect) {
        events.emit(GameEvent::PowerupCollected { entity, effect });
    } else if let Some(kind) = world.get::<&Pickup>(entity).ok().map(|p| p.kind) {
        events.emit(GameEvent::PickupCollected { entity, kind });
    }
}
