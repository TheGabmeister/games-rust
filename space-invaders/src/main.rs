mod game;

use game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::prelude::*;

const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_FRAME_DT: f32 = 0.25;

fn window_conf() -> Conf {
    Conf {
        window_title: "Space Invaders".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    let mut accumulator = 0.0;

    loop {
        let frame_dt = get_frame_time().min(MAX_FRAME_DT);
        accumulator += frame_dt;

        while accumulator >= FIXED_DT {
            game.update_fixed(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        game.draw();
        next_frame().await;
    }
}
