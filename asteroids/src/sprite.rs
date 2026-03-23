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
        self.draw_tinted(t, WHITE);
    }

    pub fn draw_tinted(&self, t: &Transform, color: Color) {
        let w = self.texture.width()  * t.scale;
        let h = self.texture.height() * t.scale;
        draw_texture_ex(
            &self.texture,
            t.x - w / 2.0,
            t.y - h / 2.0,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                rotation:  t.rot,
                pivot:     Some(vec2(t.x, t.y)),
                ..Default::default()
            },
        );
    }

    /// Half-dimensions after scaling, useful for screen-wrapping.
    pub fn half_size(&self, scale: f32) -> (f32, f32) {
        (self.texture.width() * scale / 2.0, self.texture.height() * scale / 2.0)
    }
}
