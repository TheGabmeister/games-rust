use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::ecs::Velocity;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub movement: Vec2,
    pub blip_pressed: bool,
}

pub fn sample_input() -> InputState {
    let mut movement = Vec2::ZERO;

    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        movement.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        movement.x += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        movement.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        movement.y += 1.0;
    }

    if movement.length_squared() > 1.0 {
        movement = movement.normalize();
    }

    InputState {
        movement,
        blip_pressed: is_key_pressed(KeyCode::Space),
    }
}

pub fn apply_player_velocity(world: &mut World, player: Entity, input: InputState, speed: f32) {
    if let Ok(mut velocity) = world.get::<&mut Velocity>(player) {
        velocity.value = input.movement * speed;
    }
}
