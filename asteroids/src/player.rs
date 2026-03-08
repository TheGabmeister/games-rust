use macroquad::prelude::*;

use crate::collidable::Collidable;
use crate::input::InputState;

const ROTATE_SPEED: f32 = 3.0;  // radians per second
const THRUST:       f32 = 250.0; // pixels per second²
const BRAKE:        f32 = 3.0;   // damping factor when S is held
const MAX_SPEED:    f32 = 400.0;

pub struct Player {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    pub lives: u32,
    pub angle: f32, // radians, 0 = facing up
    vx:        f32,
    vy:        f32,
    texture:   Texture2D,
}

impl Player {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            x:       screen_width()  / 2.0,
            y:       screen_height() / 2.0,
            alive:   true,
            lives:   3,
            angle:   0.0,
            vx:      0.0,
            vy:      0.0,
            texture,
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if input.move_left  { self.angle -= ROTATE_SPEED * dt; }
        if input.move_right { self.angle += ROTATE_SPEED * dt; }

        if input.move_up {
            self.vx += self.angle.sin() * THRUST * dt;
            self.vy -= self.angle.cos() * THRUST * dt;
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

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        let hw = self.texture.width()  / 2.0;
        let hh = self.texture.height() / 2.0;
        let sw = screen_width();
        let sh = screen_height();

        if self.x - hw > sw  { self.x = -hw; }
        if self.x + hw < 0.0 { self.x = sw + hw; }
        if self.y - hh > sh  { self.y = -hh; }
        if self.y + hh < 0.0 { self.y = sh + hh; }
    }

    pub fn draw(&self) {
        let hw = self.texture.width()  / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture_ex(
            &self.texture,
            self.x - hw,
            self.y - hh,
            WHITE,
            DrawTextureParams {
                rotation: self.angle,
                pivot: Some(vec2(self.x, self.y)),
                ..Default::default()
            },
        );
    }
}

impl Collidable for Player {
    fn collider(&self) -> Rect {
        let hw = self.texture.width()  / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }
}
