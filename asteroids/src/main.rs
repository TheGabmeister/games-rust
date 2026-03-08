use macroquad::prelude::*;

mod game;
use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Template".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        game.update();
        if game.should_quit {
            break;
        }
        game.draw();
        next_frame().await;
    }
}
