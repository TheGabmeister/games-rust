use macroquad::prelude::*;

use crate::collidable::Collidable;

pub struct Laser {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    vx:        f32,
    vy:        f32,
    angle:     f32, // radians, 0 = pointing up
    texture:   Texture2D,
}

impl Laser {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, texture: Texture2D) -> Self {
        Self { x, y, vx, vy, angle: vx.atan2(-vy), texture, alive: true }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        let w = self.texture.width();
        let h = self.texture.height();
        if self.x < -w || self.x > screen_width()  + w
        || self.y < -h || self.y > screen_height() + h {
            self.alive = false;
        }
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

impl Collidable for Laser {
    fn collider(&self) -> Rect {
        let w = self.texture.width();
        let h = self.texture.height();
        let cos_a = self.angle.cos().abs();
        let sin_a = self.angle.sin().abs();
        let rw = w * cos_a + h * sin_a;
        let rh = w * sin_a + h * cos_a;
        Rect::new(self.x - rw / 2.0, self.y - rh / 2.0, rw, rh)
    }
}
