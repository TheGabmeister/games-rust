use macroquad::prelude::*;

use crate::resources::{GameMode, Resources};

pub(super) fn read_input(resources: &mut Resources) {
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    resources.input.move_axis = match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    resources.input.fire_pressed = is_key_pressed(KeyCode::Space);
    resources.input.start_pressed = is_key_pressed(KeyCode::Enter);
    resources.input.pause_pressed = is_key_pressed(KeyCode::P);
}

pub(super) fn toggle_pause(resources: &mut Resources) {
    if resources.flow.mode == GameMode::Pause {
        resources.flow.mode = resources.flow.mode_before_pause;
        resources.flow.mode_timer = 0.0;
    } else if !matches!(resources.flow.mode, GameMode::Attract | GameMode::GameOver) {
        resources.flow.mode_before_pause = resources.flow.mode;
        resources.flow.mode = GameMode::Pause;
        resources.flow.mode_timer = 0.0;
    }
}
