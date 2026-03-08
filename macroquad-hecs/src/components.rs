use macroquad::prelude::*;

// --- Spatial ---
pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

// --- Stats ---
pub struct Health(pub i32);
pub struct Speed(pub f32);
pub struct Damage(pub i32);
pub struct KillCount(pub i32);

// --- Rendering ---
pub struct Tint(pub Color);

// --- Tags ---
pub struct Player;
