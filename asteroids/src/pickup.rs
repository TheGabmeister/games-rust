use macroquad::prelude::*;

use crate::collidable::Collidable;
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Pickup {
    pub transform: Transform,
    pub alive:     bool,
    sprite:        Sprite,
}

impl Pickup {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self {
            transform: Transform::new(x, y),
            alive:     true,
            sprite:    Sprite::new(texture),
        }
    }

    pub fn draw(&self) {
        self.sprite.draw(&self.transform);
    }
}

impl Collidable for Pickup {
    fn collider(&self) -> Rect {
        self.sprite.collider(&self.transform)
    }
}
