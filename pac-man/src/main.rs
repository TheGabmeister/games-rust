mod entities;
mod game;
mod input;
mod maze;
mod render;

use macroquad::prelude::{get_frame_time, next_frame, Conf};

use game::Game;

fn window_conf() -> Conf {
    render::window_conf()
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
        game.update(dt);
        render::draw(&game);
        next_frame().await;
    }
}
