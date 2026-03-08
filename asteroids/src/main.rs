#![windows_subsystem = "windows"]
#![cfg_attr(debug_assertions, allow(unused))] // Warn user of unused code during Release builds.

use macroquad::audio::{play_sound, PlaySoundParams};
use macroquad::prelude::*;

mod assets;
mod collidable;
mod enemy;
mod game;
mod laser;
mod pickup;
mod player;

use assets::Assets;
use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load().await;

    play_sound(&assets.music_spaceshooter, PlaySoundParams { looped: true, volume: 1.0 });

    let mut game = Game::new(&assets);

    loop {
        game.update(get_frame_time());
        game.draw();

        if game.should_quit {
            break;
        }

        next_frame().await
    }
}
