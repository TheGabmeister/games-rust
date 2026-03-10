use std::collections::HashMap;

use macroquad::miniquad::EventHandler;
use macroquad::prelude::{Texture2D, Vec2};

use crate::assets::LoadedAssets;
use crate::components::TextureId;
use crate::constants::PLAYER_START_LIVES;
use crate::events::EventBus;
use crate::managers::{GameDirector, MusicManager, SfxManager};

// ---------------------------------------------------------------------------
// Resources root — grouped by domain to avoid god-object growth.
// ---------------------------------------------------------------------------

pub struct Resources {
    pub textures: Textures,
    pub sfx: SfxManager,
    pub music: MusicManager,
    pub director: GameDirector,
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
            sfx: SfxManager::new(sfx),
            music: MusicManager::new(music),
            director: GameDirector::default(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Won,
    Lost,
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
