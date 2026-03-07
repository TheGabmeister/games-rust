use hecs::World;
use macroquad::prelude::*;

use crate::assets::TextureId;

#[derive(Clone)]
pub struct Name(pub String);

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
}

#[derive(Clone, Copy)]
pub struct PreviousTransform {
    pub position: Vec2,
}

#[derive(Clone, Copy)]
pub struct Velocity {
    pub value: Vec2,
}

#[derive(Clone, Copy)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Clone)]
pub struct Sprite {
    pub size: Vec2,
    pub color: Color,
    pub texture: Option<TextureId>,
    pub layer: RenderLayer,
    pub space: RenderSpace,
}

/// Tag — marks the player entity. Systems find the player via query instead of
/// relying on a stored `Entity` handle.
#[derive(Clone, Copy)]
pub struct Player;

/// Tag — entity bounces off world edges.
#[derive(Clone, Copy)]
pub struct Bouncer;

/// Tracks the player's collision state. Lives on the player entity so systems
/// can read/write it via ECS queries rather than external bookkeeping fields.
#[derive(Clone, Default)]
pub struct CollisionState {
    pub is_colliding: bool,
    /// True only during the frame the collision first started.
    pub started_colliding: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderLayer(pub i16);

impl RenderLayer {
    pub const ACTOR: Self = Self(100);
    pub const FOREGROUND: Self = Self(200);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderSpace {
    World,
    Screen,
}

pub fn spawn_template_entities(world: &mut World) {
    world.spawn((
        Name("Player".to_owned()),
        Transform {
            position: vec2(90.0, 120.0),
        },
        PreviousTransform {
            position: vec2(90.0, 120.0),
        },
        Velocity { value: Vec2::ZERO },
        Collider {
            size: vec2(48.0, 48.0),
        },
        Sprite {
            size: vec2(48.0, 48.0),
            color: SKYBLUE,
            texture: Some(TextureId::Player),
            layer: RenderLayer::ACTOR,
            space: RenderSpace::World,
        },
        Player,
        CollisionState::default(),
    ));

    world.spawn((
        Name("MovingEnemy".to_owned()),
        Transform {
            position: vec2(420.0, 200.0),
        },
        PreviousTransform {
            position: vec2(420.0, 200.0),
        },
        Velocity {
            value: vec2(130.0, 100.0),
        },
        Collider {
            size: vec2(56.0, 56.0),
        },
        Sprite {
            size: vec2(56.0, 56.0),
            color: ORANGE,
            texture: Some(TextureId::Enemy),
            layer: RenderLayer::ACTOR,
            space: RenderSpace::World,
        },
        Bouncer,
    ));

    world.spawn((
        Name("Wall".to_owned()),
        Transform {
            position: vec2(250.0, 320.0),
        },
        Collider {
            size: vec2(320.0, 34.0),
        },
        Sprite {
            size: vec2(320.0, 34.0),
            color: DARKGRAY,
            texture: None,
            layer: RenderLayer::FOREGROUND,
            space: RenderSpace::World,
        },
    ));
}
