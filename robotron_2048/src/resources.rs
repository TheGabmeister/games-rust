use macroquad::audio::{Sound, load_sound};
use macroquad::prelude::*;

use crate::components::{EnemyKind, TextureId};

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

impl InputState {
    /// Keep continuous controls but clear one-frame button edges for
    /// additional fixed-timestep simulation steps in the same render frame.
    pub fn fixed_step_continuation(self) -> Self {
        Self {
            shoot_pressed: false,
            confirm_pressed: false,
            cancel_pressed: false,
            resume_pressed: false,
            debug_toggle_pressed: false,
            ..self
        }
    }
}

#[derive(Clone, Copy)]
pub struct WaveSpawnSpec {
    pub kind: EnemyKind,
    pub count: usize,
}

#[derive(Clone, Copy)]
pub struct WaveDefinition {
    pub entries: &'static [WaveSpawnSpec],
}

const WAVE_1: [WaveSpawnSpec; 1] = [WaveSpawnSpec {
    kind: EnemyKind::Grunt,
    count: 2,
}];
const WAVE_2: [WaveSpawnSpec; 2] = [
    WaveSpawnSpec {
        kind: EnemyKind::Grunt,
        count: 2,
    },
    WaveSpawnSpec {
        kind: EnemyKind::Enforcer,
        count: 2,
    },
];
const WAVE_3: [WaveSpawnSpec; 3] = [
    WaveSpawnSpec {
        kind: EnemyKind::Grunt,
        count: 2,
    },
    WaveSpawnSpec {
        kind: EnemyKind::Hulk,
        count: 2,
    },
    WaveSpawnSpec {
        kind: EnemyKind::Enforcer,
        count: 3,
    },
];
const WAVE_4: [WaveSpawnSpec; 3] = [
    WaveSpawnSpec {
        kind: EnemyKind::Grunt,
        count: 2,
    },
    WaveSpawnSpec {
        kind: EnemyKind::Hulk,
        count: 3,
    },
    WaveSpawnSpec {
        kind: EnemyKind::Enforcer,
        count: 4,
    },
];

pub const WAVE_DEFINITIONS: [WaveDefinition; 4] = [
    WaveDefinition { entries: &WAVE_1 },
    WaveDefinition { entries: &WAVE_2 },
    WaveDefinition { entries: &WAVE_3 },
    WaveDefinition { entries: &WAVE_4 },
];

pub struct WaveSpawnRequest {
    pub difficulty_cycle: usize,
    pub definition: &'static WaveDefinition,
}

pub struct WaveDirector {
    current_wave: usize,
    spawn_pending: bool,
}

impl WaveDirector {
    pub fn new() -> Self {
        Self {
            current_wave: 0,
            spawn_pending: true,
        }
    }

    pub fn reset(&mut self) {
        self.current_wave = 0;
        self.spawn_pending = true;
    }

    pub fn wave_number(&self) -> usize {
        self.current_wave + 1
    }

    pub fn queue_next_wave(&mut self) {
        self.current_wave += 1;
        self.spawn_pending = true;
    }

    pub fn consume_spawn_request(&mut self) -> Option<WaveSpawnRequest> {
        if !self.spawn_pending {
            return None;
        }

        self.spawn_pending = false;
        let def_index = self.current_wave % WAVE_DEFINITIONS.len();
        let difficulty_cycle = self.current_wave / WAVE_DEFINITIONS.len();
        Some(WaveSpawnRequest {
            difficulty_cycle,
            definition: &WAVE_DEFINITIONS[def_index],
        })
    }
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
    pub player_died: bool,
    pub audio_queue: Vec<SoundId>,
    pub input: InputState,
    pub debug_enabled: bool,
    pub wave_director: WaveDirector,
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
            player_died: false,
            audio_queue: Vec::new(),
            input: InputState::default(),
            debug_enabled: false,
            wave_director: WaveDirector::new(),
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
