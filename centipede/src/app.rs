use macroquad::prelude::*;

use crate::audio::AudioSystem;
use crate::domain::{GameConfig, World};
use crate::input::read_frame_input;
use crate::render::{Renderer, UiOverlayState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Title,
    Playing,
    GameOver,
}

pub struct GameApp {
    state: AppState,
    renderer: Renderer,
    audio: AudioSystem,
    world: World,
    config: GameConfig,
    fixed_accumulator: f32,
    high_score: u32,
    seed_counter: u64,
}

impl GameApp {
    pub async fn new() -> Self {
        let config = GameConfig::default();
        let world = World::new(config.clone(), 0xC0FFEE);

        Self {
            state: AppState::Title,
            renderer: Renderer::new(config.clone()),
            audio: AudioSystem::new().await,
            world,
            config,
            fixed_accumulator: 0.0,
            high_score: 0,
            seed_counter: 0xABCDEF,
        }
    }

    pub fn frame(&mut self) -> bool {
        let frame_input = read_frame_input();
        if frame_input.quit_pressed {
            return true;
        }

        match self.state {
            AppState::Title => {
                if frame_input.start_pressed {
                    self.restart_world();
                    self.state = AppState::Playing;
                }
            }
            AppState::Playing => {
                self.fixed_accumulator += get_frame_time().clamp(0.0, 0.25);
                while self.fixed_accumulator >= self.config.fixed_dt {
                    self.world
                        .update(self.config.fixed_dt, frame_input.gameplay);
                    self.fixed_accumulator -= self.config.fixed_dt;
                }
                self.audio.consume(&self.world.emit_events());
                self.high_score = self.high_score.max(self.world.score);

                if self.world.is_game_over() {
                    self.state = AppState::GameOver;
                }
            }
            AppState::GameOver => {
                if frame_input.start_pressed {
                    self.restart_world();
                    self.state = AppState::Playing;
                }
            }
        }

        let overlay = match self.state {
            AppState::Title => UiOverlayState::Title,
            AppState::Playing => UiOverlayState::Playing,
            AppState::GameOver => UiOverlayState::GameOver,
        };
        self.renderer.draw(&self.world, overlay, self.high_score);
        false
    }

    fn restart_world(&mut self) {
        self.seed_counter = self
            .seed_counter
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        self.world = World::new(self.config.clone(), self.seed_counter);
        self.fixed_accumulator = 0.0;
    }
}
