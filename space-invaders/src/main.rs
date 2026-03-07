mod game;

use game::{Game, GameState, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::{
    audio::{PlaySoundParams, load_sound, play_sound, stop_sound},
    prelude::*,
};

const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_FRAME_DT: f32 = 0.25;
const MUSIC_PATH: &str = "assets/music_spaceshooter.ogg";
const MUSIC_VOLUME: f32 = 0.45;

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
    let background_music = load_sound(MUSIC_PATH)
        .await
        .expect("failed to load assets/music_spaceshooter.ogg");
    let mut music_is_playing = false;

    loop {
        let frame_dt = get_frame_time().min(MAX_FRAME_DT);
        accumulator += frame_dt;

        while accumulator >= FIXED_DT {
            game.update_fixed(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        let should_play_music = matches!(game.state, GameState::Playing);
        if should_play_music && !music_is_playing {
            play_sound(
                &background_music,
                PlaySoundParams {
                    looped: true,
                    volume: MUSIC_VOLUME,
                },
            );
            music_is_playing = true;
        } else if !should_play_music && music_is_playing {
            stop_sound(&background_music);
            music_is_playing = false;
        }

        game.draw();
        next_frame().await;
    }
}
