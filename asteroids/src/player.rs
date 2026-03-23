use macroquad::prelude::*;

use crate::box_collider::BoxCollider;
use crate::collidable::{Collider, Collidable};
use crate::input::InputState;
use crate::sprite::Sprite;
use crate::transform::Transform;

const ROTATE_SPEED: f32 = 3.0;   // radians per second
const THRUST:       f32 = 250.0; // pixels per second²
const BRAKE:        f32 = 3.0;   // damping factor when S is held
const MAX_SPEED:    f32 = 400.0;

pub struct Player {
    pub transform:    Transform,
    pub alive:        bool,
    pub lives:        u32,
    sprite:           Sprite,
    box_collider:     BoxCollider,
    vx:               f32,
    vy:               f32,
    invincible_timer: f32,
}

impl Player {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            transform:    Transform::new(screen_width() / 2.0, screen_height() / 2.0),
            alive:        true,
            lives:        3,
            sprite:       Sprite::new(texture),
            box_collider: BoxCollider::default(),
            vx:           0.0,
            vy:           0.0,
            invincible_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }

        if input.move_left  { self.transform.rot -= ROTATE_SPEED * dt; }
        if input.move_right { self.transform.rot += ROTATE_SPEED * dt; }

        if input.move_up {
            self.vx += self.transform.rot.sin() * THRUST * dt;
            self.vy -= self.transform.rot.cos() * THRUST * dt;
        }
        if input.move_down {
            let damping = (1.0 - BRAKE * dt).clamp(0.0, 1.0);
            self.vx *= damping;
            self.vy *= damping;
        }

        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed > MAX_SPEED {
            self.vx = self.vx / speed * MAX_SPEED;
            self.vy = self.vy / speed * MAX_SPEED;
        }

        self.transform.x += self.vx * dt;
        self.transform.y += self.vy * dt;

        let (hw, hh) = self.sprite.half_size(self.transform.scale);
        self.transform.wrap_screen(hw, hh);
    }

    pub fn respawn(&mut self) {
        self.transform.x   = screen_width()  / 2.0;
        self.transform.y   = screen_height() / 2.0;
        self.transform.rot = 0.0;
        self.vx            = 0.0;
        self.vy            = 0.0;
        self.alive         = true;
        self.invincible_timer = 2.0;
    }

    pub fn is_invincible(&self) -> bool {
        self.invincible_timer > 0.0
    }

    pub fn draw(&self) {
        if self.invincible_timer > 0.0 {
            // Blink: skip drawing every other 0.1s interval
            if (self.invincible_timer * 10.0) as i32 % 2 == 0 {
                return;
            }
        }
        self.sprite.draw(&self.transform);
    }

}

impl Collidable for Player {
    fn collider(&self) -> Collider {
        Collider::Obb(self.box_collider.obb(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        ))
    }
}
