use macroquad::prelude::*;

use crate::collidable::Collidable;

pub struct Pickup {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    texture:   Texture2D,
}

impl Pickup {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self { x, y, texture, alive: true }
    }

    pub fn draw(&self) {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}

impl Collidable for Pickup {
    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }
}
