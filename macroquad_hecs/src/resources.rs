use std::collections::HashMap;

use macroquad::miniquad::EventHandler;
use macroquad::prelude::{Texture2D, Vec2};

use crate::assets::LoadedAssets;
use crate::audio::{MusicManager, SfxManager};
use crate::components::TextureId;
use crate::constants::PLAYER_START_LIVES;
use crate::events::EventBus;

// ---------------------------------------------------------------------------
// Resources root — grouped by domain to avoid god-object growth.
// ---------------------------------------------------------------------------

pub struct Resources {
    pub textures: Textures,
    pub audio: AudioState,
    pub state: GameState,
    pub input: InputState,
    pub events: EventBus,
}

impl Resources {
    pub fn new(assets: LoadedAssets) -> Self {
        let LoadedAssets {
            textures,
            sfx,
            music,
        } = assets;

        Self {
            textures: Textures { textures },
            audio: AudioState {
                sfx: SfxManager::new(sfx),
                music: MusicManager::new(music),
            },
            state: GameState::default(),
            input: InputState::default(),
            events: EventBus::default(),
        }
    }
}

pub struct Textures {
    textures: HashMap<TextureId, Texture2D>,
}

impl Textures {
    /// Borrow a texture by ID. Panics if the texture was not loaded.
    pub fn texture(&self, id: TextureId) -> &Texture2D {
        self.textures
            .get(&id)
            .unwrap_or_else(|| panic!("Texture {id:?} not loaded"))
    }
}

pub struct AudioState {
    pub sfx: SfxManager,
    pub music: MusicManager,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    Won,
    Lost,
}

pub struct GameState {
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub phase: GamePhase,
    pub debug_mode: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: PLAYER_START_LIVES,
            high_score: 0,
            phase: GamePhase::Playing,
            debug_mode: false,
        }
    }
}

impl GameState {
    pub fn reset_run(&mut self) {
        self.score = 0;
        self.lives = PLAYER_START_LIVES;
        self.phase = GamePhase::Playing;
    }

    pub fn update_high_score(&mut self) {
        self.high_score = self.high_score.max(self.score);
    }

    pub fn add_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
    }

    pub fn add_lives_clamped(&mut self, amount: u32, max_lives: u32) {
        self.lives = self.lives.saturating_add(amount).min(max_lives);
    }
}

// ---------------------------------------------------------------------------
// Input state — snapshot captured once per frame.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub move_axis: Vec2,
    pub fire_held: bool,
    pub confirm_pressed: bool,
    pub cancel_pressed: bool,
    pub debug_toggle_pressed: bool,
}
