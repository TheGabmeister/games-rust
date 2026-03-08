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

    /// Draws the oriented (rotated) bounding box for debug visualization.
    pub fn draw_debug(&self, t: &Transform, base_w: f32, base_h: f32, color: Color) {
        let hw = base_w * t.scale * self.x_scale / 2.0;
        let hh = base_h * t.scale * self.y_scale / 2.0;
        let cos_r = t.rot.cos();
        let sin_r = t.rot.sin();
        let corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)];
        let pts: [Vec2; 4] = corners.map(|(cx, cy)| vec2(
            t.x + cx * cos_r - cy * sin_r,
            t.y + cx * sin_r + cy * cos_r,
        ));
        for i in 0..4 {
            let j = (i + 1) % 4;
            draw_line(pts[i].x, pts[i].y, pts[j].x, pts[j].y, 1.5, color);
        }
    }
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self { x_scale: 1.0, y_scale: 1.0 }
    }
}
