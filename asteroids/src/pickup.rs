use macroquad::prelude::*;

use crate::box_collider::BoxCollider;
use crate::collidable::{Collider, Collidable};
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Pickup {
    pub transform: Transform,
    pub alive:     bool,
    sprite:        Sprite,
    box_collider:  BoxCollider,
}

impl Pickup {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self {
            transform:    Transform::new(x, y),
            alive:        true,
            sprite:       Sprite::new(texture),
            box_collider: BoxCollider::default(),
        }
    }

    pub fn draw(&self) {
        self.sprite.draw(&self.transform);
    }

}

impl Collidable for Pickup {
    fn collider(&self) -> Collider {
        Collider::Obb(self.box_collider.obb(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        ))
    }
}
