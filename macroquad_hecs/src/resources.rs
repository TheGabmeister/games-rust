use std::collections::HashMap;

use macroquad::audio::Sound;
use macroquad::prelude::{Texture2D, Vec2};

use crate::assets::LoadedAssets;
use crate::components::TextureId;
use crate::events::{EventBus, MusicCommand, MusicId, SfxId};

// ---------------------------------------------------------------------------
// Resources — central shared state passed to all systems.
// ---------------------------------------------------------------------------

pub struct Resources {
    // Asset storage (private — access via texture() / sfx() / music())
    textures: HashMap<TextureId, Texture2D>,
    sfx: HashMap<SfxId, Sound>,
    music: HashMap<MusicId, Sound>,

    /// One-shot sound effects to play this frame; drained by system_sfx.
    pub sfx_queue: Vec<SfxId>,

    /// Music commands to process this frame; drained by system_music.
    pub music_queue: Vec<MusicCommand>,

    /// Currently playing music track, if any.
    pub current_music: Option<MusicId>,

    /// Global music volume [0.0, 1.0].
    pub music_volume: f32,

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
            sfx: assets.sfx,
            music: assets.music,
            sfx_queue: Vec::new(),
            music_queue: Vec::new(),
            current_music: None,
            music_volume: 0.4,
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

    /// Borrow a sound effect by ID. Panics if the SFX was not loaded.
    pub fn sfx(&self, id: SfxId) -> &Sound {
        self.sfx
            .get(&id)
            .unwrap_or_else(|| panic!("SFX {id:?} not loaded"))
    }

    /// Borrow a music track by ID. Panics if the track was not loaded.
    pub fn music(&self, id: MusicId) -> &Sound {
        self.music
            .get(&id)
            .unwrap_or_else(|| panic!("Music track {id:?} not loaded"))
    }

    /// Queue an SFX to be played this frame by system_sfx.
    pub fn queue_sfx(&mut self, id: SfxId) {
        self.sfx_queue.push(id);
    }

    /// Queue a music command to be handled by system_music.
    pub fn queue_music(&mut self, command: MusicCommand) {
        self.music_queue.push(command);
    }

    /// Convenience helper for queuing a music-play command.
    pub fn play_music(&mut self, id: MusicId) {
        self.queue_music(MusicCommand::Play(id));
    }

    /// Convenience helper for queuing a music-stop command.
    pub fn stop_music(&mut self) {
        self.queue_music(MusicCommand::Stop);
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
