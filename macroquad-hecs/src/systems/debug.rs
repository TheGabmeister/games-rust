use hecs::World;
use macroquad::prelude::*;

use crate::components::{Collider, Position};

/// Draw collider wireframes for all entities that have Position + Collider.
pub fn system_draw_colliders(world: &World) {
    for (pos, collider) in world.query::<(&Position, &Collider)>().iter() {
        match collider {
            Collider::Circle { radius } => {
                draw_circle_lines(pos.0.x, pos.0.y, *radius, 1.5, LIME);
            }
            Collider::Box { half_extents } => {
                let x = pos.0.x - half_extents.x;
                let y = pos.0.y - half_extents.y;
                let w = half_extents.x * 2.0;
                let h = half_extents.y * 2.0;
                draw_rectangle_lines(x, y, w, h, 1.5, ORANGE);
            }
        }
    }
}
