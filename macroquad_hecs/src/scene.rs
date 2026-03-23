use hecs::World;
use macroquad::prelude::*;
use serde::Deserialize;

use crate::components::{EnemyKind, PickupKind, PowerupEffect, Transform};
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
    Enemy {
        kind: EnemyKind,
        pos: (f32, f32),
        #[serde(default)]
        rot: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
    },
    Pickup {
        kind: PickupKind,
        pos: (f32, f32),
        #[serde(default)]
        rot: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
    },
    Powerup {
        effect: PowerupEffect,
        pos: (f32, f32),
        #[serde(default)]
        rot: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
    },
    OldHero {
        pos: (f32, f32),
        #[serde(default)]
        rot: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
    },
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
    let options = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    options.from_str(text)
        .unwrap_or_else(|e| panic!("Failed to parse scene file '{}': {}", path, e))
}

/// Apply optional rot/scale overrides from an EntityDef onto a spawned entity's Transform.
fn apply_transform_overrides(
    world: &mut World,
    entity: hecs::Entity,
    rot: Option<f32>,
    scale: Option<f32>,
) {
    if rot.is_some() || scale.is_some() {
        let mut t = world.get::<&mut Transform>(entity).unwrap();
        if let Some(r) = rot {
            t.rot = r;
        }
        if let Some(s) = scale {
            t.scale = s;
        }
    }
}

/// Spawn a single EntityDef, returning the entity handle.
fn spawn_entity_def(
    world: &mut World,
    res: &Resources,
    entity_def: &EntityDef,
) -> hecs::Entity {
    match entity_def {
        EntityDef::Enemy { kind, pos, .. } => {
            prefabs::spawn_enemy(world, *kind, vec2(pos.0, pos.1))
        }
        EntityDef::Pickup { kind, pos, .. } => {
            prefabs::spawn_pickup(world, *kind, vec2(pos.0, pos.1))
        }
        EntityDef::Powerup { effect, pos, .. } => {
            prefabs::spawn_powerup(world, *effect, vec2(pos.0, pos.1))
        }
        EntityDef::OldHero { pos, .. } => {
            prefabs::spawn_old_hero(world, &res.anim_db, vec2(pos.0, pos.1))
        }
    }
}

/// Extract the optional rot/scale from any EntityDef variant.
fn entity_def_overrides(entity_def: &EntityDef) -> (Option<f32>, Option<f32>) {
    match entity_def {
        EntityDef::Enemy { rot, scale, .. }
        | EntityDef::Pickup { rot, scale, .. }
        | EntityDef::Powerup { rot, scale, .. }
        | EntityDef::OldHero { rot, scale, .. } => (*rot, *scale),
    }
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
        let entity = spawn_entity_def(world, res, entity_def);
        let (rot, scale) = entity_def_overrides(entity_def);
        apply_transform_overrides(world, entity, rot, scale);
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
        let entity = spawn_entity_def(world, res, entity_def);
        let (rot, scale) = entity_def_overrides(entity_def);
        apply_transform_overrides(world, entity, rot, scale);
    }

    if let Some(id) = scene_def.music {
        res.events.emit(PlayMusic { id });
    }
}
