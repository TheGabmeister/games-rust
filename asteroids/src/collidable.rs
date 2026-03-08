use macroquad::prelude::*;

use crate::box_collider::Obb;
use crate::circle_collider::Circle;

pub enum Collider {
    Obb(Obb),
    Circle(Circle),
}

impl Collider {
    pub fn overlaps(&self, other: &Collider) -> bool {
        match (self, other) {
            (Collider::Obb(a),    Collider::Obb(b))    => a.overlaps(b),
            (Collider::Circle(a), Collider::Circle(b)) => circle_vs_circle(a, b),
            (Collider::Obb(o),    Collider::Circle(c)) => circle_vs_obb(c, o),
            (Collider::Circle(c), Collider::Obb(o))    => circle_vs_obb(c, o),
        }
    }
}

pub trait Collidable {
    fn collider(&self) -> Collider;
}

pub fn overlaps(a: &impl Collidable, b: &impl Collidable) -> bool {
    a.collider().overlaps(&b.collider())
}

pub fn draw_debug(entity: &impl Collidable, color: Color) {
    match entity.collider() {
        Collider::Obb(obb) => {
            let pts = obb.corners();
            for i in 0..4 {
                let j = (i + 1) % 4;
                draw_line(pts[i].x, pts[i].y, pts[j].x, pts[j].y, 1.5, color);
            }
        }
        Collider::Circle(c) => {
            draw_circle_lines(c.cx, c.cy, c.r, 1.5, color);
        }
    }
}

fn circle_vs_circle(a: &Circle, b: &Circle) -> bool {
    let dx = a.cx - b.cx;
    let dy = a.cy - b.cy;
    let r_sum = a.r + b.r;
    dx * dx + dy * dy <= r_sum * r_sum
}

fn circle_vs_obb(c: &Circle, o: &Obb) -> bool {
    // Transform circle centre into OBB local space, then clamp to box extents.
    let dx = c.cx - o.cx;
    let dy = c.cy - o.cy;
    let cos_r = o.rot.cos();
    let sin_r = o.rot.sin();
    let local_x = dx * cos_r + dy * sin_r;
    let local_y = -dx * sin_r + dy * cos_r;
    let dist_x = local_x - local_x.clamp(-o.hw, o.hw);
    let dist_y = local_y - local_y.clamp(-o.hh, o.hh);
    dist_x * dist_x + dist_y * dist_y <= c.r * c.r
}
