use macroquad::prelude::*;
use hecs::World;
use ::rand::RngExt;

use crate::components::*;

const PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];

pub fn batch_spawn_entities(world: &mut World, n: usize) {
    let mut rng = ::rand::rng();
    let to_spawn = (0..n).map(|_| {
        let pos    = Position(vec2(rng.random_range(0.0..800.0), rng.random_range(0.0..600.0)));
        let vel    = Velocity(Vec2::ZERO);
        let speed  = Speed(rng.random_range(50.0..200.0));
        let hp     = Health(rng.random_range(30..50));
        let dmg    = Damage(rng.random_range(1..10));
        let kc     = KillCount(0);
        let tint   = PALETTE[rng.random_range(0..PALETTE.len())];
        let sprite = Sprite { texture: TextureId::EnemyBlack, tint };
        (pos, vel, speed, hp, dmg, kc, Enemy, sprite, DrawLayer(LAYER_ENEMY))
    });
    world.spawn_batch(to_spawn);
}

/// Player no longer carries Texture2D directly — the Sprite component holds a
/// TextureId and `system_draw` resolves it through Resources at render time.
pub fn spawn_player(world: &mut World) {
    world.spawn((
        Position(vec2(400.0, 300.0)),
        Velocity(Vec2::ZERO),
        Player,
        Sprite { texture: TextureId::PlayerShip, tint: WHITE },
        DrawLayer(LAYER_PLAYER),
    ));
}
