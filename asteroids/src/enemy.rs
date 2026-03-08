use macroquad::prelude::*;

pub struct Enemy {
    pub x:     f32,
    pub y:     f32,
    pub alive: bool,
    texture:   Texture2D,
    shoot_timer: f32,
}

impl Enemy {
    pub fn new(texture: Texture2D) -> Self {
        Self {
            x:           100.0,
            y:           100.0,
            texture,
            alive:       true,
            shoot_timer: 5.0,
        }
    }

    /// Returns `true` when it's time to fire.
    pub fn update(&mut self, dt: f32) -> bool {
        self.shoot_timer -= dt;
        if self.shoot_timer <= 0.0 {
            self.shoot_timer = 5.0;
            return true;
        }
        false
    }

    pub fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
    }

    pub fn draw(&self) {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}
