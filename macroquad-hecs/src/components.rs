use macroquad::prelude::*;
use hecs::Entity;

// --- Spatial ---
pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

// --- Stats ---
pub struct Speed(pub f32);
pub struct Lifetime(pub f32); // seconds until auto-despawn

// --- Rendering ---
#[derive(Clone, Copy)]
pub enum TextureId {
    PlayerShip,
    EnemyBlack,
    PlayerLaser,
}

pub struct Sprite {
    pub texture: TextureId,
    pub tint: Color,
}

pub struct DrawLayer(pub u8);

pub const LAYER_PROJECTILE: u8 = 1;
pub const LAYER_ENEMY: u8 = 2;
pub const LAYER_PLAYER: u8 = 3;

// --- Collision ---
#[derive(Clone, Copy)]
pub enum Collider {
    Circle { radius: f32 },
    Box { half_extents: Vec2 },
}

// --- Tags ---
pub struct Player;
pub struct Enemy;

// --- Relationships ---
pub struct Projectile {
    pub owner: Entity,
}
