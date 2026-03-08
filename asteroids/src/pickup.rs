use macroquad::prelude::*;

use crate::box_collider::BoxCollider;
use crate::collidable::Collidable;
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

    pub fn draw_collider_debug(&self, color: Color) {
        self.box_collider.draw_debug(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
            color,
        );
    }
}

impl Collidable for Pickup {
    fn collider(&self) -> Rect {
        self.box_collider.rect(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        )
    }
}
