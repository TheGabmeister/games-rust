use macroquad::prelude::*;

use crate::entities::{Direction, Pacman};

pub(crate) fn apply_pacman_input(pacman: &mut Pacman) {
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        pacman.mover.desired_dir = Direction::Left;
    } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        pacman.mover.desired_dir = Direction::Right;
    } else if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        pacman.mover.desired_dir = Direction::Up;
    } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        pacman.mover.desired_dir = Direction::Down;
    }
}
