use std::collections::HashSet;

use hecs::Entity;
use macroquad::prelude::Vec2;

use crate::anim_manifest;
use crate::events::{EventQueue, EventRegistry};
use crate::managers::{AnimationDb, Assets, GameDirector, MusicManager, SfxManager};

// ---------------------------------------------------------------------------
// Resources root — grouped by domain to avoid god-object growth.
// ---------------------------------------------------------------------------

pub struct Resources {
    pub assets: Assets,
    pub anim_db: AnimationDb,
    pub sfx: SfxManager,
    pub music: MusicManager,
    pub director: GameDirector,
    pub input: InputState,
    pub events: EventQueue,
    pub event_registry: EventRegistry,
    pub despawns: DespawnQueue,
}

impl Resources {
    pub fn new(assets: Assets) -> Self {
        let anim_db = anim_manifest::build_animation_db();
        let sfx = SfxManager::new(&assets);
        let music = MusicManager::new(&assets);
        Self {
            assets,
            anim_db,
            sfx,
            music,
            director: GameDirector::default(),
            input: InputState::default(),
            events: EventQueue::default(),
            event_registry: EventRegistry::default(),
            despawns: DespawnQueue::default(),
        }
    }
}

#[derive(Default)]
pub struct DespawnQueue {
    entities: HashSet<Entity>,
}

impl DespawnQueue {
    pub fn extend<I>(&mut self, entities: I)
    where
        I: IntoIterator<Item = Entity>,
    {
        self.entities.extend(entities);
    }

    pub fn drain(&mut self) -> Vec<Entity> {
        self.entities.drain().collect()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
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
    pub nav_up_pressed: bool,
    pub nav_down_pressed: bool,
    pub debug_toggle_pressed: bool,
}

impl InputState {
    pub fn clear_transients(&mut self) {
        self.confirm_pressed = false;
        self.cancel_pressed = false;
        self.nav_up_pressed = false;
        self.nav_down_pressed = false;
        self.debug_toggle_pressed = false;
    }
}
