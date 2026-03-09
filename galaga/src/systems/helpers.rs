use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    EnemyKind, PathFollower, PathKind, Player, RenderablePrimitive, Timer, Transform,
};

pub(super) fn set_path(
    world: &mut World,
    entity: Entity,
    points: Vec<Vec2>,
    duration: f32,
    kind: PathKind,
) {
    let _ = world.remove_one::<PathFollower>(entity);
    let _ = world.insert_one(
        entity,
        PathFollower {
            points,
            duration,
            t: 0.0,
            kind,
            finished: false,
        },
    );
    let _ = world.remove_one::<Timer>(entity);
    let _ = world.insert_one(entity, Timer { remaining: 0.0 });
}

pub(super) fn player_position(world: &World) -> Option<Vec2> {
    world
        .query::<(&Player, &Transform)>()
        .iter()
        .next()
        .map(|(_, transform)| transform.pos)
}

pub(super) fn enemy_appearance(kind: EnemyKind) -> (f32, RenderablePrimitive) {
    match kind {
        EnemyKind::Bee => (
            12.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Circle,
                size: vec2(22.0, 22.0),
                color: GOLD,
                layer: 30,
            },
        ),
        EnemyKind::Butterfly => (
            12.5,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(24.0, 24.0),
                color: PINK,
                layer: 30,
            },
        ),
        EnemyKind::BossGalaga => (
            15.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Rect,
                size: vec2(30.0, 22.0),
                color: RED,
                layer: 32,
            },
        ),
        EnemyKind::GalaxianFlagship => (
            13.5,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(28.0, 26.0),
                color: GREEN,
                layer: 31,
            },
        ),
        EnemyKind::CapturedFighter => (
            13.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(24.0, 24.0),
                color: ORANGE,
                layer: 31,
            },
        ),
    }
}
