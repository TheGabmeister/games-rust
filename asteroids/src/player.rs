use macroquad::prelude::*;

use crate::collidable::Collidable;
use crate::input::InputState;

pub struct Player {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    pub lives: u32,
    speed:     f32,
    texture:   Texture2D,
}

impl Player {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            x:       screen_width() / 2.0,
            y:       screen_height() / 2.0,
            speed:   200.0,
            texture,
            alive:   true,
            lives:   3,
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if input.move_left  { self.x -= self.speed * dt; }
        if input.move_right { self.x += self.speed * dt; }
        if input.move_up    { self.y -= self.speed * dt; }
        if input.move_down  { self.y += self.speed * dt; }

        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        self.x = self.x.clamp(hw, screen_width() - hw);
        self.y = self.y.clamp(hh, screen_height() - hh);
    }

    pub fn draw(&self) {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}

impl Collidable for Player {
    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }
}
