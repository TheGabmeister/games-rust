use macroquad::prelude::*;

use crate::collidable::Collidable;
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Laser {
    pub transform: Transform,
    pub alive:     bool,
    sprite:        Sprite,
    vx:            f32,
    vy:            f32,
}

impl Laser {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, texture: Texture2D) -> Self {
        let mut transform = Transform::new(x, y);
        transform.rot = vx.atan2(-vy);
        Self { transform, vx, vy, sprite: Sprite::new(texture), alive: true }
    }

    pub fn update(&mut self, dt: f32) {
        self.transform.x += self.vx * dt;
        self.transform.y += self.vy * dt;
        let w = self.sprite.texture.width();
        let h = self.sprite.texture.height();
        if self.transform.x < -w || self.transform.x > screen_width()  + w
        || self.transform.y < -h || self.transform.y > screen_height() + h {
            self.alive = false;
        }
    }

    pub fn draw(&self) {
        self.sprite.draw(&self.transform);
    }
}

impl Collidable for Laser {
    fn collider(&self) -> Rect {
        self.sprite.collider(&self.transform)
    }
}
