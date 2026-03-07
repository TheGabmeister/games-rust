use macroquad::prelude::*;
use crate::assets::AssetManager;
use crate::debug::is_debug;
use crate::entities::Entity;

pub fn draw_entity(entity: &Entity, assets: &AssetManager) {
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
    if is_debug() {
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
