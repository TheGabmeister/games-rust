use hecs::World;
use macroquad::prelude::*;

use crate::components::{DrawLayer, Sprite, TextureId, Transform};
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::resources::{GameState, GameManager, Textures};

/// Draw all entities that have Transform + Sprite + DrawLayer, sorted back-to-front.
pub fn draw(world: &World, textures: &Textures) {
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
        let tex = textures.texture(texture_id);
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
pub fn draw_hud(state: &GameManager) {
    draw_text(&format!("SCORE: {}", state.score), 10.0, 24.0, 22.0, WHITE);
    draw_text(&format!("LIVES: {}", state.lives), 10.0, 50.0, 22.0, WHITE);
    draw_text(
        &format!("BEST: {}", state.high_score),
        SCREEN_WIDTH - 160.0,
        24.0,
        22.0,
        Color::from_hex(0xffd700),
    );

    let overlay = match state.phase {
        GameState::Playing => None,
        GameState::Won => Some(("STAGE CLEARED - PRESS ENTER", Color::from_hex(0x8cff8c))),
        GameState::Lost => Some(("GAME OVER - PRESS ENTER", Color::from_hex(0xff6b6b))),
    };

    if let Some((text, color)) = overlay {
        let dim = measure_text(text, None, 36, 1.0);
        let panel_h = 80.0;
        let panel_y = SCREEN_HEIGHT * 0.5 - panel_h * 0.5;
        draw_rectangle(
            0.0,
            panel_y,
            SCREEN_WIDTH,
            panel_h,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );
        draw_text(
            text,
            (SCREEN_WIDTH - dim.width) * 0.5,
            panel_y + 52.0,
            36.0,
            color,
        );
    }
}
