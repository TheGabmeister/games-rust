use macroquad::prelude::*;

use crate::box_collider::Obb;

pub trait Collidable {
    fn collider(&self) -> Obb;
}

pub fn overlaps(a: &impl Collidable, b: &impl Collidable) -> bool {
    a.collider().overlaps(&b.collider())
}

pub fn draw_debug(entity: &impl Collidable, color: Color) {
    let pts = entity.collider().corners();
    for i in 0..4 {
        let j = (i + 1) % 4;
        draw_line(pts[i].x, pts[i].y, pts[j].x, pts[j].y, 1.5, color);
    }
}
