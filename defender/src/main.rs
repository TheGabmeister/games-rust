use macroquad::prelude::*;

mod constants;
mod world;
mod terrain;
mod player;
mod enemies;
mod astronauts;
mod bullets;
mod particles;
mod scanner;
mod collision;
mod scoring;
mod audio;
mod game;

use game::Game;
use constants::MAX_DT;

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
