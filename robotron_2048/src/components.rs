use hecs::Entity;
use macroquad::prelude::*;

// --- Spatial ---
pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

// --- Stats ---
pub struct Speed(pub f32);
pub struct Health(pub i32);
pub struct HitSlow(pub f32); // seconds remaining for temporary movement slowdown
pub struct Lifetime(pub f32); // seconds until auto-despawn

// --- Enemy Capabilities ---
#[derive(Clone, Copy)]
pub struct Chase {
    pub steer_accel: f32,
    pub forward_weight: f32,
    pub strafe_weight: f32,
    pub jitter_weight: f32,
    pub hit_slow_multiplier: f32,
}

#[derive(Clone, Copy)]
pub struct RangedAttack {
    pub remaining: f32,
    pub period: f32,
    pub projectile_kind: ProjectileKind,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub projectile_radius: f32,
    pub projectile_damage: i32,
    pub projectile_texture: TextureId,
    pub projectile_tint: Color,
    pub aim_jitter_rad: f32,
}

#[derive(Clone, Copy)]
pub struct Spawner {
    pub remaining: f32,
    pub period: f32,
    pub spawn_kind: EnemyKind,
    pub burst_count: u32,
    pub spawn_radius: f32,
}

#[derive(Clone, Copy)]
pub struct ContactDamage {
    pub damage: i32,
}

#[derive(Clone, Copy)]
pub struct HitReaction {
    pub hit_slow_seconds: f32,
    pub knockback_speed: f32,
}

pub struct WaveClearTarget;

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
