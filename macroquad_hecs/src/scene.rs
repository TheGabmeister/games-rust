use hecs::World;
use macroquad::prelude::*;
use serde::Deserialize;

use crate::components::{EnemyKind, PickupKind, PowerupEffect};
use crate::events::{MusicId, PlayMusic};
use crate::prefabs;
use crate::resources::{GameState, Resources};

// ---------------------------------------------------------------------------
// Scene data — deserialized from .ron files
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SceneDef {
    pub name: String,
    pub music: Option<MusicId>,
    pub background_color: Option<(f32, f32, f32, f32)>,
    pub entities: Vec<EntityDef>,
}

#[derive(Deserialize)]
pub enum EntityDef {
    Enemy { kind: EnemyKind, pos: (f32, f32) },
    Pickup { kind: PickupKind, pos: (f32, f32) },
    Powerup { effect: PowerupEffect, pos: (f32, f32) },
    OldHero { pos: (f32, f32) },
}

// ---------------------------------------------------------------------------
// Active scene — which "screen" the game is on
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveScene {
    Menu,
    Gameplay,
    CharacterDemo,
}

// ---------------------------------------------------------------------------
// Scene loading & spawning
// ---------------------------------------------------------------------------

pub fn load_scene_def(path: &str) -> SceneDef {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|e| panic!("Failed to read scene file '{}': {}", path, e));
    let text = std::str::from_utf8(&bytes)
        .unwrap_or_else(|e| panic!("Scene file '{}' is not valid UTF-8: {}", path, e));
    ron::from_str(text)
        .unwrap_or_else(|e| panic!("Failed to parse scene file '{}': {}", path, e))
}

/// Despawn everything, spawn entities from the current campaign scene, and start playing.
pub fn enter_gameplay_scene(
    world: &mut World,
    res: &mut Resources,
    scene_def: &SceneDef,
) {
    world.clear();
    res.events.drain_raw();
    res.despawns.clear();

    prefabs::spawn_player(world);

    for entity_def in &scene_def.entities {
        match entity_def {
            EntityDef::Enemy { kind, pos } => {
                prefabs::spawn_enemy(world, *kind, vec2(pos.0, pos.1));
            }
            EntityDef::Pickup { kind, pos } => {
                prefabs::spawn_pickup(world, *kind, vec2(pos.0, pos.1));
            }
            EntityDef::Powerup { effect, pos } => {
                prefabs::spawn_powerup(world, *effect, vec2(pos.0, pos.1));
            }
            EntityDef::OldHero { pos } => {
                prefabs::spawn_old_hero(world, &res.anim_db, vec2(pos.0, pos.1));
            }
        }
    }

    if let Some(id) = scene_def.music {
        res.events.emit(PlayMusic { id });
    }

    res.director.state = GameState::Playing;
}

/// Spawn entities from a scene without player or gameplay state — used for demo/showcase scenes.
pub fn enter_character_scene(
    world: &mut World,
    res: &mut Resources,
    scene_def: &SceneDef,
) {
    world.clear();
    res.events.drain_raw();
    res.despawns.clear();

    for entity_def in &scene_def.entities {
        match entity_def {
            EntityDef::Enemy { kind, pos } => {
                prefabs::spawn_enemy(world, *kind, vec2(pos.0, pos.1));
            }
            EntityDef::Pickup { kind, pos } => {
                prefabs::spawn_pickup(world, *kind, vec2(pos.0, pos.1));
            }
            EntityDef::Powerup { effect, pos } => {
                prefabs::spawn_powerup(world, *effect, vec2(pos.0, pos.1));
            }
            EntityDef::OldHero { pos } => {
                prefabs::spawn_old_hero(world, &res.anim_db, vec2(pos.0, pos.1));
            }
        }
    }

    if let Some(id) = scene_def.music {
        res.events.emit(PlayMusic { id });
    }
}
