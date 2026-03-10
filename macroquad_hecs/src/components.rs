use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
    pub rot: f32
}

pub struct BoxCollider {
    pub size: Vec2,
}

// --- Rendering ---
#[derive(Clone, Copy)]
pub enum TextureId {
    PlayerShip,
    PlayerLaser,
    Enemy,
    Item,
    Powerup,
}

pub struct Sprite {
    pub texture: TextureId,
    pub tint: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Enemy1,
    Enemy2,
    Enemy3,
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
}