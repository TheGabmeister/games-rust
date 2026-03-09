use macroquad::prelude::*;

mod astronauts;
mod audio;
mod bullets;
mod collision;
mod constants;
mod enemies;
mod game;
mod particles;
mod player;
mod scanner;
mod scoring;
mod terrain;
mod world;

use constants::MAX_DT;
use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Defender".to_string(),
        window_width: 960,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(MAX_DT);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
