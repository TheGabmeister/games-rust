use hecs::World;
use macroquad::prelude::*;

use crate::components::{BoxCollider, CircleCollider, Transform};

/// Draw collider wireframes for all entities that have a collider component.
pub fn system_draw_colliders(world: &World) {
    // Box colliders (green)
    for (transform, col) in world.query::<(&Transform, &BoxCollider)>().iter() {
        let x = transform.pos.x - col.half.x;
        let y = transform.pos.y - col.half.y;
        let w = col.half.x * 2.0;
        let h = col.half.y * 2.0;
        draw_rectangle_lines(x, y, w, h, 1.5, GREEN);
    }

    // Circle colliders (yellow)
    for (transform, col) in world.query::<(&Transform, &CircleCollider)>().iter() {
        draw_circle_lines(transform.pos.x, transform.pos.y, col.radius, 1.5, YELLOW);
    }
}
