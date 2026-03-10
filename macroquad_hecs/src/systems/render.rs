use hecs::World;
use macroquad::prelude::*;

use crate::components::{DrawLayer, Sprite, TextureId, Transform};
use crate::constants::{SCREEN_WIDTH};
use crate::resources::Resources;

/// Draw all entities that have Transform + Sprite + DrawLayer, sorted back-to-front.
pub fn draw(world: &World, res: &Resources) {
    clear_background(Color::from_hex(0x0a0a1a));

    // Collect drawables: (layer, pos, rot, texture_id, tint)
    let mut drawables: Vec<(DrawLayer, Vec2, f32, TextureId, Color)> = world
        .query::<(&DrawLayer, &Transform, &Sprite)>()
        .iter()
        .map(|(layer, transform, sprite)| {
            (*layer, transform.pos, transform.rot, sprite.texture, sprite.tint)
        })
        .collect();

    // Sort by DrawLayer (derives Ord — lower value = drawn first = behind)
    drawables.sort_unstable_by_key(|(layer, ..)| *layer);

    for (_, pos, rot, texture_id, tint) in drawables {
        let tex = res.texture(texture_id);
        let w = tex.width();
        let h = tex.height();

        draw_texture_ex(
            tex,
            pos.x - w * 0.5,
            pos.y - h * 0.5,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                rotation: rot,
                ..Default::default()
            },
        );
    }
}

/// Overlay HUD (score, lives, high score).
pub fn draw_hud(res: &Resources) {
    draw_text(&format!("SCORE: {}", res.score), 10.0, 24.0, 22.0, WHITE);
    draw_text(&format!("LIVES: {}", res.lives), 10.0, 50.0, 22.0, WHITE);
    draw_text(
        &format!("BEST: {}", res.high_score),
        SCREEN_WIDTH - 160.0,
        24.0,
        22.0,
        Color::from_hex(0xffd700),
    );
}
