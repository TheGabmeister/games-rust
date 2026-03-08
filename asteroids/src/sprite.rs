use macroquad::prelude::*;

use crate::transform::Transform;

pub struct Sprite {
    pub texture: Texture2D,
}

impl Sprite {
    pub fn new(texture: Texture2D) -> Self {
        Self { texture }
    }

    pub fn draw(&self, t: &Transform) {
        let w = self.texture.width()  * t.scale;
        let h = self.texture.height() * t.scale;
        draw_texture_ex(
            &self.texture,
            t.x - w / 2.0,
            t.y - h / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                rotation:  t.rot,
                pivot:     Some(vec2(t.x, t.y)),
                ..Default::default()
            },
        );
    }

    /// Axis-aligned bounding box of the (possibly rotated, scaled) sprite.
    pub fn collider(&self, t: &Transform) -> Rect {
        let w = self.texture.width()  * t.scale;
        let h = self.texture.height() * t.scale;
        let cos_a = t.rot.cos().abs();
        let sin_a = t.rot.sin().abs();
        let rw = w * cos_a + h * sin_a;
        let rh = w * sin_a + h * cos_a;
        Rect::new(t.x - rw / 2.0, t.y - rh / 2.0, rw, rh)
    }

    /// Half-dimensions after scaling, useful for screen-wrapping.
    pub fn half_size(&self, scale: f32) -> (f32, f32) {
        (self.texture.width() * scale / 2.0, self.texture.height() * scale / 2.0)
    }
}
