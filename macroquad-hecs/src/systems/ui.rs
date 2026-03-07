use hecs::World;
use macroquad::prelude::*;

use crate::assets::AssetManager;
use crate::ecs::{CollisionState, Player};

/// Draws the in-game HUD. Reads collision notes from the player's `CollisionState`
/// component and asset warnings directly from `AssetManager`.
pub fn draw_hud(world: &World, assets: &AssetManager) {
    draw_text(
        "Move: WASD/Arrows | Pause: P/Esc | Toggle Debug: F3 | Blip: Space",
        16.0,
        28.0,
        24.0,
        WHITE,
    );

    let mut q = world.query::<(&Player, &CollisionState)>();
    if let Some((_, state)) = q.iter().next() {
        let mut text_y = 56.0;
        for note in &state.notes {
            draw_text(note, 16.0, text_y, 24.0, YELLOW);
            text_y += 24.0;
        }
    }

    if let Some(error) = assets.critical_error() {
        draw_text(error, 16.0, screen_height() - 22.0, 20.0, RED);
    }

    let mut warning_y = screen_height() - 48.0;
    for warning in assets.warnings().iter().take(2) {
        draw_text(warning, 16.0, warning_y, 20.0, ORANGE);
        warning_y -= 24.0;
    }
}

pub fn draw_paused_overlay() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.55),
    );

    let title = "PAUSED";
    let title_size = 64.0;
    let title_width = measure_text(title, None, title_size as u16, 1.0).width;
    draw_text(
        title,
        (screen_width() - title_width) * 0.5,
        screen_height() * 0.5 - 10.0,
        title_size,
        WHITE,
    );
    draw_text(
        "Press P or Esc to resume",
        screen_width() * 0.5 - 140.0,
        screen_height() * 0.5 + 28.0,
        28.0,
        LIGHTGRAY,
    );
}
