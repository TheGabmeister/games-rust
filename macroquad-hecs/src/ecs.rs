use hecs::{Entity, World};
use macroquad::prelude::*;

pub const PLAYER_SPEED: f32 = 260.0;

#[derive(Clone)]
pub struct Name(pub String);

#[derive(Clone, Copy)]
pub struct Transform {
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
    pub texture: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Bouncer;

pub fn spawn_template_entities(world: &mut World) -> Entity {
    let player = world.spawn((
        Name("Player".to_owned()),
        Transform {
            position: vec2(90.0, 120.0),
        },
        Velocity { value: Vec2::ZERO },
        Collider {
            size: vec2(48.0, 48.0),
        },
        Sprite {
            size: vec2(48.0, 48.0),
            color: SKYBLUE,
            texture: Some("player".to_owned()),
        },
    ));

    world.spawn((
        Name("MovingEnemy".to_owned()),
        Transform {
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
            texture: Some("enemy".to_owned()),
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
        },
    ));

    player
}
