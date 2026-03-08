use hecs::World;
use macroquad::prelude::*;

use crate::components::*;

pub fn spawn_player(world: &mut World) {
    world.spawn((
        Position(vec2(400.0, 300.0)),
        Velocity(Vec2::ZERO),
        Player,
        Sprite {
            texture: TextureId::PlayerShip,
            tint: WHITE,
        },
        DrawLayer(LAYER_PLAYER),
        Collider::Circle { radius: 16.0 },
    ));
}
