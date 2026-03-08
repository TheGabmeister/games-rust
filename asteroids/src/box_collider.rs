use macroquad::prelude::*;

use crate::transform::Transform;

pub struct BoxCollider {
    pub x_scale: f32,
    pub y_scale: f32,
}

impl BoxCollider {
    pub fn new(x_scale: f32, y_scale: f32) -> Self {
        Self { x_scale, y_scale }
    }

    /// Axis-aligned bounding box of the (possibly rotated) collider.
    /// base_w / base_h are the sprite's unscaled texture dimensions.
    pub fn rect(&self, t: &Transform, base_w: f32, base_h: f32) -> Rect {
        let w = base_w * t.scale * self.x_scale;
        let h = base_h * t.scale * self.y_scale;
        let cos_a = t.rot.cos().abs();
        let sin_a = t.rot.sin().abs();
        let rw = w * cos_a + h * sin_a;
        let rh = w * sin_a + h * cos_a;
        Rect::new(t.x - rw / 2.0, t.y - rh / 2.0, rw, rh)
    }
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self { x_scale: 1.0, y_scale: 1.0 }
    }
}
