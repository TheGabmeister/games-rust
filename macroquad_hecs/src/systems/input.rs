use hecs::*;
use macroquad::prelude::*;

use crate::components::*;
use crate::resources::{InputState, Resources};

/// Capture key/mouse input once per frame into a singleton resource.
pub fn system_capture_input(input: &mut InputState) {
    let mut move_axis = Vec2::ZERO;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        move_axis.x += 1.0;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        move_axis.x -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        move_axis.y += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        move_axis.y -= 1.0;
    }

    input.confirm_pressed = is_key_pressed(KeyCode::Enter);
    input.cancel_pressed = is_key_pressed(KeyCode::Escape);
    input.debug_toggle_pressed = is_key_pressed(KeyCode::F1);
}