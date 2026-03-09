use macroquad::prelude::*;

use crate::domain::CommandInput;

#[derive(Debug, Clone, Copy)]
pub struct FrameInput {
    pub gameplay: CommandInput,
    pub start_pressed: bool,
    pub quit_pressed: bool,
}

pub fn read_frame_input() -> FrameInput {
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    let up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
    let down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);

    let mut axis = vec2(0.0, 0.0);
    if left {
        axis.x -= 1.0;
    }
    if right {
        axis.x += 1.0;
    }
    if up {
        axis.y -= 1.0;
    }
    if down {
        axis.y += 1.0;
    }
    if axis.length_squared() > 1.0 {
        axis = axis.normalize();
    }

    FrameInput {
        gameplay: CommandInput {
            move_axis: axis,
            fire: is_key_down(KeyCode::Space),
        },
        start_pressed: is_key_pressed(KeyCode::Enter),
        quit_pressed: is_key_pressed(KeyCode::Escape),
    }
}
