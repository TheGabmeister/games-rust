use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::collidable::Collidable;

pub struct Asteroid {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    vx:        f32,
    vy:        f32,
    texture:   Texture2D,
}

impl Asteroid {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        let angle = gen_range(0.0f32, std::f32::consts::TAU);
        let speed = gen_range(60.0f32, 140.0);
        Self {
            x,
            y,
            alive: true,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            texture,
        }
    }

    pub fn update(&mut self, dt: f32) {
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
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}

impl Collidable for Asteroid {
    fn collider(&self) -> Rect {
        let hw = self.texture.width()  / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }
}
