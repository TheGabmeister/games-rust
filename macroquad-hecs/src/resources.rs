use macroquad::audio::{play_sound, stop_sound, PlaySoundParams};
use macroquad::prelude::Texture2D;

use crate::assets::Assets;
use crate::components::TextureId;

// ── Game state machine ────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

// ── Audio ─────────────────────────────────────────────────────────────────────

/// Sound identifiers pushed into the queue by systems each frame.
/// `system_audio` drains the queue once per frame and plays each sound.
/// This decouples logic systems from the audio API.
#[derive(Clone, Copy)]
pub enum SoundId {
    Laser,
    Bump,
    Lose,
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// Game-wide singleton state.  Lives outside the ECS world because hecs is
/// optimised for many same-shaped entities, not per-frame global state.
pub struct Resources {
    pub assets:      Assets,
    pub state:       GameState,
    pub score:       u32,
    pub audio_queue: Vec<SoundId>,
}

impl Resources {
    pub fn new(assets: Assets) -> Self {
        Self {
            assets,
            state:       GameState::MainMenu,
            score:       0,
            audio_queue: Vec::new(),
        }
    }

    /// Look up the actual GPU texture from a TextureId.
    pub fn texture(&self, id: TextureId) -> &Texture2D {
        match id {
            TextureId::PlayerShip  => &self.assets.player_ship,
            TextureId::EnemyBlack  => &self.assets.enemy_black,
            TextureId::PlayerLaser => &self.assets.player_laser,
        }
    }

    /// Queue a one-shot sound for `system_audio` to play this frame.
    pub fn queue_sound(&mut self, id: SoundId) {
        self.audio_queue.push(id);
    }

    pub fn start_music(&self) {
        play_sound(
            &self.assets.music_spaceshooter,
            PlaySoundParams { looped: true, volume: 0.4 },
        );
    }

    pub fn stop_music(&self) {
        stop_sound(&self.assets.music_spaceshooter);
    }
}
