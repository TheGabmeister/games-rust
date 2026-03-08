use macroquad::prelude::*;

use crate::box_collider::BoxCollider;
use crate::collidable::Collidable;
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Enemy {
    pub transform:   Transform,
    pub alive:       bool,
    sprite:          Sprite,
    box_collider:    BoxCollider,
    shoot_timer:     f32,
}

impl Enemy {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            transform:    Transform::new(100.0, 100.0),
            alive:        true,
            sprite:       Sprite::new(texture),
            box_collider: BoxCollider::default(),
            shoot_timer:  5.0,
        }
    }

    /// Returns `true` when it's time to fire.
    pub fn update(&mut self, dt: f32) -> bool {
        self.shoot_timer -= dt;
        if self.shoot_timer <= 0.0 {
            self.shoot_timer = 5.0;
            return true;
        }
        false
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

impl Collidable for Enemy {
    fn collider(&self) -> Rect {
        self.box_collider.rect(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        )
    }
}
