pub const SCREEN_WIDTH: f32 = 600.0;
pub const SCREEN_HEIGHT: f32 = 800.0;

pub const FIXED_DT: f32 = 1.0 / 60.0;
pub const MAX_FRAME_TIME: f32 = 0.25;

pub const ASSETS_DIR: &str = "assets";

// ---------------------------------------------------------------------------
// Collision layers (bitmask)
// ---------------------------------------------------------------------------
pub const LAYER_PLAYER: u32 = 1 << 0;
pub const LAYER_PLAYER_BULLET: u32 = 1 << 1;
pub const LAYER_ENEMY: u32 = 1 << 2;
pub const LAYER_ENEMY_BULLET: u32 = 1 << 3;
pub const LAYER_PICKUP: u32 = 1 << 4;

// ---------------------------------------------------------------------------
// Draw layers (lower = drawn first = behind)
// ---------------------------------------------------------------------------
pub const DRAW_PICKUP: u8 = 10;
pub const DRAW_ENEMY: u8 = 20;
pub const DRAW_PLAYER: u8 = 30;
pub const DRAW_BULLET: u8 = 40;

// ---------------------------------------------------------------------------
// Player tuning
// ---------------------------------------------------------------------------
pub const PLAYER_SPEED: f32 = 200.0;
pub const PLAYER_FIRE_RATE: f32 = 0.15;
pub const PLAYER_BOLT_FIRE_RATE: f32 = 0.07;
pub const PLAYER_BULLET_SPEED: f32 = 500.0;
pub const PLAYER_START_X: f32 = SCREEN_WIDTH * 0.5;
pub const PLAYER_START_Y: f32 = SCREEN_HEIGHT - 80.0;

// ---------------------------------------------------------------------------
// Enemy tuning
// ---------------------------------------------------------------------------
pub const ENEMY_SPEED_BLACK: f32 = 80.0;
pub const ENEMY_SPEED_BLUE: f32 = 120.0;
pub const ENEMY_SPEED_GREEN: f32 = 60.0;
pub const ENEMY_FIRE_RATE: f32 = 1.5;
pub const ENEMY_BULLET_SPEED: f32 = 250.0;

// ---------------------------------------------------------------------------
// Score
// ---------------------------------------------------------------------------
pub const SCORE_ENEMY_BLACK: u32 = 100;
pub const SCORE_ENEMY_BLUE: u32 = 150;
pub const SCORE_ENEMY_GREEN: u32 = 75;
pub const SCORE_PICKUP_STAR: u32 = 500;

// ---------------------------------------------------------------------------
// Player state
// ---------------------------------------------------------------------------
pub const PLAYER_START_LIVES: u32 = 3;
pub const PLAYER_MAX_LIVES: u32 = 5;

// ---------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------
pub const BULLET_LIFETIME: f32 = 2.0;

// ---------------------------------------------------------------------------
// Powerups
// ---------------------------------------------------------------------------
pub const POWERUP_DURATION_BOLT: f32 = 5.0;
pub const POWERUP_DURATION_SHIELD: f32 = 5.0;
