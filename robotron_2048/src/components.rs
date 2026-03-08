use macroquad::prelude::*;
use hecs::Entity;

// --- Spatial ---
pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

// --- Stats ---
pub struct Speed(pub f32);
pub struct Health(pub i32);
pub struct HitSlow(pub f32); // seconds remaining for temporary movement slowdown
pub struct Lifetime(pub f32); // seconds until auto-despawn
pub struct FireCooldown {
    pub remaining: f32,
    pub period: f32,
}
pub struct SpawnCooldown {
    pub remaining: f32,
    pub period: f32,
}

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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyKind {
    Grunt,
    Hulk,
    Brain,
    Sphereoid,
    Enforcer,
    Quark,
    Tank,
    Prog,
}
pub struct Invulnerable(pub bool);

// --- Relationships ---
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectileKind {
    PlayerLaser,
    EnforcerSpark,
    TankShell,
    CruiseMissile,
}

#[derive(Clone, Copy)]
pub struct Projectile {
    pub owner: Entity,
    pub faction: Faction,
    pub kind: ProjectileKind,
    pub damage: i32,
}
