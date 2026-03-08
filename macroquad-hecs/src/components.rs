use macroquad::prelude::*;
use hecs::Entity;

// --- Spatial ---
pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

// --- Stats ---
pub struct Speed(pub f32);
pub struct Lifetime(pub f32);    // seconds until auto-despawn

// --- Rendering ---
#[derive(Clone, Copy)]
pub enum TextureId {
    PlayerShip,
    EnemyBlack,
    PlayerLaser,
}

pub struct Sprite {
    pub texture: TextureId,
    pub tint:    Color,
}

pub struct DrawLayer(pub u8);

pub const LAYER_PROJECTILE: u8 = 1;
pub const LAYER_ENEMY:      u8 = 2;
pub const LAYER_PLAYER:     u8 = 3;

// --- Collision ---
#[derive(Clone, Copy)]
pub enum Collider {
    Circle { radius: f32 },
    Box    { half_extents: Vec2 },
}

impl Collider {
    /// Returns true if the two colliders, at their respective world positions, overlap.
    pub fn overlaps(self, pos_a: Vec2, other: Collider, pos_b: Vec2) -> bool {
        match (self, other) {
            (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
                pos_a.distance(pos_b) < r1 + r2
            }
            (Collider::Box { half_extents: h1 }, Collider::Box { half_extents: h2 }) => {
                let delta = (pos_a - pos_b).abs();
                delta.x < h1.x + h2.x && delta.y < h1.y + h2.y
            }
            // Circle vs Box (symmetric — abs() makes order irrelevant)
            (Collider::Circle { radius: r }, Collider::Box { half_extents: h })
            | (Collider::Box { half_extents: h }, Collider::Circle { radius: r }) => {
                let d = (pos_a - pos_b).abs(); // component-wise distance, always ≥ 0
                let closest = d.min(h);        // nearest point on box surface
                (d - closest).length() < r
            }
        }
    }
}

// --- Tags ---
pub struct Player;
pub struct Enemy;

// --- Relationships ---
pub struct Projectile {
    pub owner: Entity,
}
