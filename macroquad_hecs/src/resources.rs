use macroquad::prelude::Vec2;

use crate::events::EventBus;
use crate::managers::{GameDirector, LoadedAssets, MusicManager, SfxManager, TextureManager};

// ---------------------------------------------------------------------------
// Resources root — grouped by domain to avoid god-object growth.
// ---------------------------------------------------------------------------

pub struct Resources {
    pub textures: TextureManager,
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
            textures: TextureManager::new(textures),
            sfx: SfxManager::new(sfx),
            music: MusicManager::new(music),
            director: GameDirector::default(),
            input: InputState::default(),
            events: EventBus::default(),
        }
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
