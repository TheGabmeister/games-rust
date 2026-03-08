use macroquad::prelude::*;

pub struct InputState {
    pub move_left:  bool,
    pub move_right: bool,
    pub move_up:    bool,
    pub move_down:  bool,
    pub shoot:      bool,
    pub confirm:    bool,
    pub back:       bool,
    pub quit:       bool,
}

impl InputState {
    pub fn capture() -> Self {
        Self {
            move_left:  is_key_down(KeyCode::A)     || is_key_down(KeyCode::Left),
            move_right: is_key_down(KeyCode::D)     || is_key_down(KeyCode::Right),
            move_up:    is_key_down(KeyCode::W)     || is_key_down(KeyCode::Up),
            move_down:  is_key_down(KeyCode::S)     || is_key_down(KeyCode::Down),
            shoot:      is_key_pressed(KeyCode::Space),
            confirm:    is_key_pressed(KeyCode::Enter),
            back:       is_key_pressed(KeyCode::Escape),
            quit:       is_key_pressed(KeyCode::Q),
        }
    }
}
