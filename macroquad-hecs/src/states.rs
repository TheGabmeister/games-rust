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
    fn update(&mut self, ctx: &mut GameData, frame_dt: f32) -> Option<AppState> {
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::P)
            || macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Escape)
        {
            return Some(AppState::Paused);
        }

        ctx.update_playing(frame_dt);
        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();
    }
}

pub struct PausedState;

impl GameState<GameData, AppState> for PausedState {
    fn update(&mut self, _ctx: &mut GameData, _frame_dt: f32) -> Option<AppState> {
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::P)
            || macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Escape)
        {
            return Some(AppState::Playing);
        }

        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();
        ctx.draw_paused_overlay();
    }
}
