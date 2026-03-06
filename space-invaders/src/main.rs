mod game;

use game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::prelude::*;

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

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
