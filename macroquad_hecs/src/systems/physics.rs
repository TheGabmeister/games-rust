use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{BoxCollider, CircleCollider, CollisionLayer, Transform};
use crate::constants::*;
use crate::events::GameEvent;
use crate::resources::Resources;

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
// Main collision system
// ---------------------------------------------------------------------------

/// Tests all collidable entity pairs and emits events via res.events.
/// Uses two-pass (snapshot → test) to avoid borrow conflicts with hecs.
pub fn system_collision(world: &mut World, res: &mut Resources) {
    // Pass 1: snapshot collidable entities (releases QueryBorrow)
    let mut aabbs: Vec<AabbEntry> = world
        .query::<(Entity, &Transform, &BoxCollider, &CollisionLayer)>()
        .iter()
        .map(|(entity, t, c, l)| AabbEntry {
            entity,
            pos: t.pos,
            half: c.half,
            layer: *l,
        })
        .collect();

    let mut circles: Vec<CircleEntry> = world
        .query::<(Entity, &Transform, &CircleCollider, &CollisionLayer)>()
        .iter()
        .map(|(entity, t, c, l)| CircleEntry {
            entity,
            pos: t.pos,
            radius: c.radius,
            layer: *l,
        })
        .collect();

    // Pass 2: test pairs and emit events
    // AABB vs AABB
    for i in 0..aabbs.len() {
        for j in (i + 1)..aabbs.len() {
            let a = &aabbs[i];
            let b = &aabbs[j];
            if layers_interact(a.layer, b.layer)
                && aabb_overlaps(a.pos, a.half, b.pos, b.half)
            {
                emit_event(a.entity, a.layer, b.entity, b.layer, res);
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
                emit_event(a.entity, a.layer, b.entity, b.layer, res);
            }
        }
    }

    // Circle vs AABB
    for c in &circles {
        for a in &aabbs {
            if layers_interact(c.layer, a.layer)
                && circle_aabb_overlaps(c.pos, c.radius, a.pos, a.half)
            {
                emit_event(c.entity, c.layer, a.entity, a.layer, res);
            }
        }
    }
}

fn layers_interact(a: CollisionLayer, b: CollisionLayer) -> bool {
    (a.mask & b.member) != 0 || (b.mask & a.member) != 0
}

fn emit_event(
    ea: Entity,
    la: CollisionLayer,
    eb: Entity,
    lb: CollisionLayer,
    res: &mut Resources,
) {
    // Identify roles by layer membership
    let a_is_player_bullet = (la.member & LAYER_PLAYER_BULLET) != 0;
    let b_is_player_bullet = (lb.member & LAYER_PLAYER_BULLET) != 0;
    let a_is_enemy_bullet = (la.member & LAYER_ENEMY_BULLET) != 0;
    let b_is_enemy_bullet = (lb.member & LAYER_ENEMY_BULLET) != 0;
    let a_is_pickup = (la.member & LAYER_PICKUP) != 0;
    let b_is_pickup = (lb.member & LAYER_PICKUP) != 0;

    if a_is_player_bullet {
        res.events.emit(GameEvent::BulletHitEnemy { bullet: ea, enemy: eb });
    } else if b_is_player_bullet {
        res.events.emit(GameEvent::BulletHitEnemy { bullet: eb, enemy: ea });
    } else if a_is_enemy_bullet {
        res.events.emit(GameEvent::BulletHitPlayer { bullet: ea });
    } else if b_is_enemy_bullet {
        res.events.emit(GameEvent::BulletHitPlayer { bullet: eb });
    } else if a_is_pickup {
        res.events.emit(GameEvent::PickupTouched { pickup: ea });
    } else if b_is_pickup {
        res.events.emit(GameEvent::PickupTouched { pickup: eb });
    }
}
