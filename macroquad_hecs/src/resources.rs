use std::collections::HashMap;

use macroquad::audio::Sound;
use macroquad::prelude::{Texture2D, Vec2};

use crate::assets::LoadedAssets;
use crate::components::TextureId;
use crate::events::{EventBus, SoundId};

// ---------------------------------------------------------------------------
// Resources — central shared state passed to all systems.
// ---------------------------------------------------------------------------

pub struct Resources {
    // Asset storage (private — access via texture() / sound())
    textures: HashMap<TextureId, Texture2D>,
    sounds: HashMap<SoundId, Sound>,

    /// Sounds to play this frame; drained by system_audio.
    pub audio_queue: Vec<SoundId>,

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
            sounds: assets.sounds,
            audio_queue: Vec::new(),
            score: 0,
            lives: 3,
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

    /// Borrow a sound by ID. Panics if the sound was not loaded.
    pub fn sound(&self, id: SoundId) -> &Sound {
        self.sounds
            .get(&id)
            .unwrap_or_else(|| panic!("Sound {id:?} not loaded"))
    }

    /// Queue a sound to be played this frame by system_audio.
    pub fn queue_sound(&mut self, id: SoundId) {
        self.audio_queue.push(id);
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
