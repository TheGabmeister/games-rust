use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveShape {
    Rect,
    Circle,
    Triangle,
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    pub vel: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderablePrimitive {
    pub shape: PrimitiveShape,
    pub size: Vec2,
    pub color: Color,
    pub layer: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub fire_cooldown: f32,
    pub invuln_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Bee,
    Butterfly,
    BossGalaga,
    GalaxianFlagship,
    CapturedFighter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMode {
    Entering,
    Formed,
    Diving,
    Returning,
    Capturing,
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub mode: EnemyMode,
    pub hp: i32,
    pub can_morph: bool,
    pub morphed: bool,
    pub carrying_player: bool,
    pub dive_shot_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug)]
pub struct Projectile {
    pub owner: ProjectileOwner,
    pub damage: i32,
    pub barrel: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct FormationSlot {
    pub target: Vec2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathKind {
    Entry,
    Dive,
    Return,
    Challenge,
    CaptureApproach,
}

#[derive(Clone, Debug)]
pub struct PathFollower {
    pub points: Vec<Vec2>,
    pub duration: f32,
    pub t: f32,
    pub kind: PathKind,
    pub finished: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TractorBeamPhase {
    Idle,
    Telegraph,
    Active,
    Cooldown,
}

#[derive(Clone, Copy, Debug)]
pub struct TractorBeamState {
    pub phase: TractorBeamPhase,
    pub timer: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CapturedShipState;

#[derive(Clone, Copy, Debug)]
pub struct DualFighterWingman {
    pub offset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Timer {
    pub remaining: f32,
}
