use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;

use crate::components::TextureId;

const ASSETS_DIR: &str = "assets";

#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

/// Sound identifiers pushed into the queue by systems each frame.
/// `system_audio` drains the queue once per frame and plays each sound.
#[derive(Clone, Copy)]
pub enum SoundId {
    Laser,
    Bump,
    Lose,
}

/// Snapshot of player/input intent captured once per frame.
#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub move_axis: Vec2,
    pub aim_screen: Vec2,
    pub shoot_pressed: bool,
    pub confirm_pressed: bool,
    pub cancel_pressed: bool,
    pub resume_pressed: bool,
    pub debug_toggle_pressed: bool,
}

/// Game-wide singleton state. Lives outside the ECS world because hecs is
/// optimized for many same-shaped entities, not per-frame global state.
pub struct Resources {
    // --- loaded assets ---
    pub player_ship: Texture2D,
    pub enemy_black: Texture2D,
    pub player_laser: Texture2D,
    pub sfx_laser: Sound,
    pub sfx_bump: Sound,
    pub sfx_lose: Sound,
    pub music_spaceshooter: Sound,

    // --- runtime state ---
    pub state: GameState,
    pub score: u32,
    pub audio_queue: Vec<SoundId>,
    pub input: InputState,
    pub debug_enabled: bool,
}

impl Resources {
    pub async fn load() -> Self {
        Self {
            player_ship: Self::load_tex("player_ship.png").await,
            enemy_black: Self::load_tex("enemy_black.png").await,
            player_laser: Self::load_tex("player_laser.png").await,
            sfx_laser: Self::load_snd("sfx_laser1.ogg").await,
            sfx_bump: Self::load_snd("sfx_bump.ogg").await,
            sfx_lose: Self::load_snd("sfx_lose.ogg").await,
            music_spaceshooter: Self::load_snd("music_spaceshooter.ogg").await,
            state: GameState::MainMenu,
            score: 0,
            audio_queue: Vec::new(),
            input: InputState::default(),
            debug_enabled: false,
        }
    }

    async fn load_tex(file: &str) -> Texture2D {
        let path = format!("{}/{}", ASSETS_DIR, file);
        load_texture(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to load texture: {}", path))
    }

    async fn load_snd(file: &str) -> Sound {
        let path = format!("{}/{}", ASSETS_DIR, file);
        load_sound(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to load sound: {}", path))
    }

    /// Look up the actual GPU texture from a TextureId.
    pub fn texture(&self, id: TextureId) -> &Texture2D {
        match id {
            TextureId::PlayerShip => &self.player_ship,
            TextureId::EnemyBlack => &self.enemy_black,
            TextureId::PlayerLaser => &self.player_laser,
        }
    }

    /// Queue a one-shot sound for `system_audio` to play this frame.
    pub fn queue_sound(&mut self, id: SoundId) {
        self.audio_queue.push(id);
    }
}
