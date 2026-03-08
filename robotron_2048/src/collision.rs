use macroquad::prelude::Vec2;

use crate::components::Collider;

/// Returns true if two colliders overlap at their respective world positions.
pub fn overlaps(a: Collider, pos_a: Vec2, b: Collider, pos_b: Vec2) -> bool {
    match (a, b) {
        (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
            pos_a.distance(pos_b) < r1 + r2
        }
        (Collider::Box { half_extents: h1 }, Collider::Box { half_extents: h2 }) => {
            let delta = (pos_a - pos_b).abs();
            delta.x < h1.x + h2.x && delta.y < h1.y + h2.y
        }
        // Circle vs Box (symmetric because abs() makes order irrelevant).
        (Collider::Circle { radius: r }, Collider::Box { half_extents: h })
        | (Collider::Box { half_extents: h }, Collider::Circle { radius: r }) => {
            let d = (pos_a - pos_b).abs();
            let closest = d.min(h);
            (d - closest).length() < r
        }
    }
}
