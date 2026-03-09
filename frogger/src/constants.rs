// ── Grid / screen layout ──────────────────────────────────────────────────────
pub const WINDOW_W: f32 = 800.0;
pub const WINDOW_H: f32 = 600.0;

pub const TILE: f32 = 40.0;
pub const COLS: i32 = 14;
pub const ROWS: i32 = 14;
pub const OFFSET_X: f32 = 120.0; // (800 - 14*40) / 2
pub const OFFSET_Y: f32 = 20.0;  // (600 - 14*40) / 2

// ── Row indices ───────────────────────────────────────────────────────────────
pub const ROW_HOMES: i32 = 0;
pub const ROW_RIVER_TOP: i32 = 1;
pub const ROW_RIVER_BOT: i32 = 5;
pub const ROW_ROAD_TOP: i32 = 7;
pub const ROW_ROAD_BOT: i32 = 11;
pub const ROW_START: i32 = 13;

// ── Home pockets: grid column index for each pocket center (0-based) ─────────
pub const HOME_COLS: [i32; 5] = [1, 3, 5, 7, 9];
pub const HOME_WIDTH: f32 = TILE;
pub const HOME_COUNT: usize = 5;

// ── Frog start cell ───────────────────────────────────────────────────────────
pub const FROG_START_COL: i32 = 6;
pub const FROG_START_ROW: i32 = ROW_START;

// ── Timing / difficulty ───────────────────────────────────────────────────────
pub const TIMER_SECS: f32 = 30.0;
pub const SPEED_INCREMENT: f32 = 0.2;
pub const MAX_SPEED_SCALE: f32 = 3.0;
pub const HOP_DURATION: f32 = 0.12; // seconds per hop animation
pub const DEATH_ANIM_SECS: f32 = 1.2;
pub const RESPAWN_DELAY_SECS: f32 = 0.8;
pub const FLY_LIFETIME_SECS: f32 = 10.0;

// ── Starting lives ────────────────────────────────────────────────────────────
pub const START_LIVES: i32 = 3;

// ── Scoring ───────────────────────────────────────────────────────────────────
pub const SCORE_HOP: i32 = 10;
pub const SCORE_HOME: i32 = 50;
pub const SCORE_TIME_MULT: i32 = 10;
pub const SCORE_FLY: i32 = 200;
pub const SCORE_LEVEL: i32 = 1000;

// ── Frog visual size (slightly smaller than tile) ─────────────────────────────
pub const FROG_W: f32 = 30.0;
pub const FROG_H: f32 = 30.0;

// ── Wrap margins ──────────────────────────────────────────────────────────────
pub const WRAP_X_MIN: f32 = OFFSET_X - TILE;
pub const WRAP_X_MAX: f32 = OFFSET_X + COLS as f32 * TILE + TILE;
