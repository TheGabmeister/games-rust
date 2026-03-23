use macroquad::prelude::*;

use crate::resources::InputState;

/// Capture key/mouse input once per frame into the InputState resource.
pub fn system_capture_input(input: &mut InputState) {
    let mut axis = Vec2::ZERO;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        axis.x += 1.0;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        axis.x -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        axis.y += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        axis.y -= 1.0;
    }

    // Normalise so diagonal movement is not faster than cardinal.
    if axis.length_squared() > 0.0 {
        axis = axis.normalize();
    }

    input.move_axis = axis;
    input.fire_held = is_key_down(KeyCode::Space);
    input.confirm_pressed |= is_key_pressed(KeyCode::Enter);
    input.cancel_pressed |= is_key_pressed(KeyCode::Escape);
    input.debug_toggle_pressed |= is_key_pressed(KeyCode::F1);
}
