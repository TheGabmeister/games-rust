use macroquad::prelude::*;

pub struct Transform {
    pub x:     f32,
    pub y:     f32,
    pub rot:   f32, // radians, 0 = facing up
    pub scale: f32,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, rot: 0.0, scale: 1.0 }
    }

    pub fn wrap_screen(&mut self, hw: f32, hh: f32) {
        let sw = screen_width();
        let sh = screen_height();
        if self.x - hw > sw  { self.x = -hw; }
        if self.x + hw < 0.0 { self.x = sw + hw; }
        if self.y - hh > sh  { self.y = -hh; }
        if self.y + hh < 0.0 { self.y = sh + hh; }
    }
}
