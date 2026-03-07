use macroquad::prelude::Vec2;

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    pub fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self {
            min: position,
            max: position + size,
        }
    }
}

pub fn intersects(a: Aabb, b: Aabb) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}
