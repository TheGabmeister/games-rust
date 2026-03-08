use macroquad::prelude::Rect;

pub trait Collidable {
    fn collider(&self) -> Rect;
}

pub fn overlaps(a: &impl Collidable, b: &impl Collidable) -> bool {
    a.collider().overlaps(&b.collider())
}
