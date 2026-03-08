use macroquad::prelude::*;

use crate::collidable::Collidable;
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Enemy {
    pub transform:   Transform,
    pub alive:       bool,
    sprite:          Sprite,
    shoot_timer:     f32,
}

impl Enemy {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            transform:   Transform::new(100.0, 100.0),
            alive:       true,
            sprite:      Sprite::new(texture),
            shoot_timer: 5.0,
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
}

impl Collidable for Enemy {
    fn collider(&self) -> Rect {
        self.sprite.collider(&self.transform)
    }
}
