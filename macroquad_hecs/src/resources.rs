use std::collections::HashMap;

use macroquad::prelude::{Texture2D, Vec2};

use crate::assets::LoadedAssets;
use crate::audio::{MusicManager, SfxManager};
use crate::components::TextureId;
use crate::constants::PLAYER_START_LIVES;
use crate::events::EventBus;

// ---------------------------------------------------------------------------
// Resources — central shared state passed to all systems.
// ---------------------------------------------------------------------------

pub struct Resources {
    // Asset storage (private — access via texture())
    textures: HashMap<TextureId, Texture2D>,

    pub sfx_manager: SfxManager,
    pub music_manager: MusicManager,

    /// Game state
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub debug_mode: bool,

    /// Per-frame input snapshot (written by system_capture_input).
    pub input: InputState,

    /// Event bus (written by systems, drained by system_process_events).
    pub events: EventBus,
}

impl Resources {
    pub fn new(assets: LoadedAssets) -> Self {
        Self {
            textures: assets.textures,
            sfx_manager: SfxManager::new(assets.sfx),
            music_manager: MusicManager::new(assets.music),
            score: 0,
            lives: PLAYER_START_LIVES,
            high_score: 0,
            debug_mode: false,
            input: InputState::default(),
            events: EventBus::default(),
        }
    }

    /// Borrow a texture by ID. Panics if the texture was not loaded.
    pub fn texture(&self, id: TextureId) -> &Texture2D {
        self.textures
            .get(&id)
            .unwrap_or_else(|| panic!("Texture {id:?} not loaded"))
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
