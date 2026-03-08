use macroquad::prelude::*;

use crate::box_collider::BoxCollider;
use crate::collidable::Collidable;
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
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if input.move_left  { self.transform.rot -= ROTATE_SPEED * dt; }
        if input.move_right { self.transform.rot += ROTATE_SPEED * dt; }

        if input.move_up {
            self.vx += self.transform.rot.sin() * THRUST * dt;
            self.vy -= self.transform.rot.cos() * THRUST * dt;
        }
        if input.move_down {
            self.vx *= 1.0 - BRAKE * dt;
            self.vy *= 1.0 - BRAKE * dt;
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
    }

    pub fn draw(&self) {
        self.sprite.draw(&self.transform);
    }
}

impl Collidable for Player {
    fn collider(&self) -> Rect {
        self.box_collider.rect(
            &self.transform,
            self.sprite.texture.width(),
            self.sprite.texture.height(),
        )
    }
}
