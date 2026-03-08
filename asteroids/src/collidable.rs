use macroquad::prelude::*;

pub trait Collidable {
    fn collider(&self) -> Rect;
}

pub fn overlaps(a: &impl Collidable, b: &impl Collidable) -> bool {
    a.collider().overlaps(&b.collider())
}

pub fn draw_debug(entity: &impl Collidable, color: Color) {
    let r = entity.collider();
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, color);
}
