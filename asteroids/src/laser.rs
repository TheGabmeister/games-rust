use macroquad::prelude::*;

use crate::collidable::Collidable;

pub struct Laser {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    vy:        f32,
    texture:   Texture2D,
}

impl Laser {
    pub fn new(x: f32, y: f32, vy: f32, texture: Texture2D) -> Self {
        Self { x, y, vy, texture, alive: true }
    }

    pub fn update(&mut self, dt: f32) {
        self.y += self.vy * dt;
        let h = self.texture.height();
        if self.y < -h || self.y > screen_height() + h {
            self.alive = false;
        }
    }

    pub fn draw(&self) {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}

impl Collidable for Laser {
    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }
}
