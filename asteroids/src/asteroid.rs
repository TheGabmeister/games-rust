use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::circle_collider::CircleCollider;
use crate::collidable::{Collider, Collidable};
use crate::sprite::Sprite;
use crate::transform::Transform;

pub struct Asteroid {
    pub transform:    Transform,
    pub alive:        bool,
    sprite:           Sprite,
    circle_collider:  CircleCollider,
    vx:               f32,
    vy:               f32,
}

impl Asteroid {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        let angle = gen_range(0.0f32, std::f32::consts::TAU);
        let speed = gen_range(60.0f32, 140.0);
        Self {
            transform:       Transform::new(x, y),
            alive:           true,
            sprite:          Sprite::new(texture),
            circle_collider: CircleCollider::default(),
            vx:              angle.cos() * speed,
            vy:              angle.sin() * speed,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.transform.x += self.vx * dt;
        self.transform.y += self.vy * dt;

        let (hw, hh) = self.sprite.half_size(self.transform.scale);
        self.transform.wrap_screen(hw, hh);
    }

    pub fn draw(&self) {
        self.sprite.draw(&self.transform);
    }
}

impl Collidable for Asteroid {
    fn collider(&self) -> Collider {
        Collider::Circle(self.circle_collider.circle(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        ))
    }
}
