use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// Spatial
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
    pub rot: f32, // radians, 0 = up
}

impl Transform {
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            rot: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    pub linear: Vec2, // pixels/sec
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { linear: vec2(x, y) }
    }
    pub fn zero() -> Self {
        Self { linear: Vec2::ZERO }
    }
}

// ---------------------------------------------------------------------------
// Collision
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box centered on Transform.pos (half-extents).
#[derive(Clone, Copy, Debug)]
pub struct BoxCollider {
    pub half: Vec2,
}

impl BoxCollider {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            half: vec2(w * 0.5, h * 0.5),
        }
    }
}

/// Circle collider centered on Transform.pos.
#[derive(Clone, Copy, Debug)]
pub struct CircleCollider {
    pub radius: f32,
}

impl CircleCollider {
    pub fn new(r: f32) -> Self {
        Self { radius: r }
    }
}

/// Bitmask controlling which collision layers an entity belongs to and checks against.
#[derive(Clone, Copy, Debug)]
pub struct CollisionLayer {
    pub member: u32, // layers this entity is ON
    pub mask: u32,   // layers this entity checks against
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureId {
    PlayerShip,
    PlayerLaser,
    EnemyShipBlack,
    EnemyShipBlue,
    EnemyShipGreen,
    EnemyShipRed,
    EnemyLaser,
    PickupLife,
    PickupStar,
    PowerupBolt,
    PowerupShield,
}

pub struct Sprite {
    pub texture: TextureId,
    pub tint: Color,
}

impl Sprite {
    pub fn new(texture: TextureId) -> Self {
        Self {
            texture,
            tint: WHITE,
        }
    }
    pub fn tinted(texture: TextureId, tint: Color) -> Self {
        Self { texture, tint }
    }
}

/// Draw ordering: lower = rendered first = behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawLayer(pub u8);

// ---------------------------------------------------------------------------
// Entity tags
// ---------------------------------------------------------------------------

pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}

pub struct Projectile {
    pub owner: ProjectileOwner,
}

// ---------------------------------------------------------------------------
// Gameplay
// ---------------------------------------------------------------------------

/// Entity is despawned when remaining reaches 0.
#[derive(Clone, Copy, Debug)]
pub struct Lifetime {
    pub remaining: f32,
}

impl Lifetime {
    pub fn new(secs: f32) -> Self {
        Self { remaining: secs }
    }
}

/// Per-entity fire cooldown timer.
#[derive(Clone, Copy, Debug)]
pub struct FireTimer {
    pub cooldown: f32, // seconds between shots
    pub timer: f32,    // counts down; fires when <= 0
}

impl FireTimer {
    pub fn new(cooldown: f32) -> Self {
        Self {
            cooldown,
            timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Black,
    Blue,
    Green,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Life,
    Star,
}

pub struct Pickup {
    pub kind: PickupKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerupEffect {
    Bolt,   // rapid fire
    Shield, // damage immunity
}

pub struct ActivePowerup {
    pub effect: PowerupEffect,
    pub duration: f32, // remaining seconds
}

/// Score awarded when this entity is destroyed.
#[derive(Clone, Copy, Debug)]
pub struct ScoreValue(pub u32);
