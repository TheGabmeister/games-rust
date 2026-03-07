use macroquad::prelude::*;

use crate::game::GameData;
use crate::state_machine::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    Playing,
    Paused,
}

impl AppState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

pub struct PlayingState;

impl GameState<GameData, AppState> for PlayingState {
    fn update(&mut self, ctx: &mut GameData) -> Option<AppState> {
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            return Some(AppState::Paused);
        }

        ctx.update_playing();
        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();
    }
}

pub struct PausedState;

impl GameState<GameData, AppState> for PausedState {
    fn update(&mut self, _ctx: &mut GameData) -> Option<AppState> {
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            return Some(AppState::Playing);
        }

        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );

        let title = "PAUSED";
        let title_size = 64.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            (screen_width() - title_width) * 0.5,
            screen_height() * 0.5 - 10.0,
            title_size,
            WHITE,
        );
        draw_text(
            "Press P or Esc to resume",
            screen_width() * 0.5 - 140.0,
            screen_height() * 0.5 + 28.0,
            28.0,
            LIGHTGRAY,
        );
    }
}
