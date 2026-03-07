use macroquad::prelude::*;
use crate::assets::AssetManager;
use crate::entities::Entity;
use crate::state::GameState;

pub fn draw_entity(entity: &Entity, assets: &AssetManager, debug: bool) {
    if !entity.active {
        return;
    }

    match entity.texture_name.as_deref().and_then(|n| assets.texture(n)) {
        Some(texture) => {
            draw_texture_ex(
                texture,
                entity.position.x,
                entity.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(entity.size),
                    ..Default::default()
                },
            );
        }
        None => {
            // Fallback: solid magenta rectangle so missing textures are obvious.
            draw_rectangle(entity.position.x, entity.position.y, entity.size.x, entity.size.y, MAGENTA);
        }
    }

    // Hitbox overlay in debug mode.
    if debug {
        draw_rectangle_lines(
            entity.position.x,
            entity.position.y,
            entity.size.x,
            entity.size.y,
            1.5,
            GREEN,
        );
    }
}

pub fn draw_overlay(state: &GameState) {
    let (msg, color) = match state {
        GameState::Paused   => ("PAUSED", WHITE),
        GameState::GameOver => ("GAME OVER", RED),
        GameState::Playing  => return,
    };
    let sz = measure_text(msg, None, 60, 1.0);
    draw_text(msg, screen_width() / 2.0 - sz.width / 2.0, screen_height() / 2.0, 60.0, color);
}
