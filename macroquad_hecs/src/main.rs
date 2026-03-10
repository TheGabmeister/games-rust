mod components;
mod constants;
mod game;
mod resources;
mod systems;

use macroquad::prelude::*;

use constants::{FIXED_DT, MAX_FRAME_TIME, SCREEN_HEIGHT, SCREEN_WIDTH};
use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Hecs".to_string(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut game = Game::new();
    let mut accumulator = 0.0_f32;

    loop {
        let dt = get_frame_time().min(MAX_FRAME_TIME);
        accumulator += dt;

        while accumulator >= FIXED_DT {
            game.update(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        game.draw();
        next_frame().await;
    }
}