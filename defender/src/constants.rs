use macroquad::prelude::Color;

// Custom colors not defined in macroquad's prelude
pub const CYAN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const MAGENTA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

// World
pub const WORLD_WIDTH: f32 = 6000.0;
pub const WORLD_HEIGHT: f32 = 570.0; // logical play area height (below scanner)

// Scanner
pub const SCANNER_HEIGHT: f32 = 30.0;

// Terrain
pub const TERRAIN_SEGMENTS: usize = 200;
pub const TERRAIN_MIN_HEIGHT: f32 = 40.0;
pub const TERRAIN_MAX_HEIGHT: f32 = 120.0;

// Player
pub const PLAYER_SPEED: f32 = 280.0;
pub const PLAYER_VERT_SPEED: f32 = 200.0;
pub const PLAYER_BULLET_SPEED: f32 = 600.0;
pub const PLAYER_FIRE_RATE: f32 = 0.12;
pub const PLAYER_BULLET_LIFETIME: f32 = 1.2;
pub const MAX_PLAYER_BULLETS: usize = 8;
pub const SMART_BOMBS_PER_LIFE: u32 = 3;
pub const HYPERSPACE_DEATH_CHANCE: f32 = 0.10;
pub const PLAYER_HALF_W: f32 = 16.0;
pub const PLAYER_HALF_H: f32 = 8.0;
pub const PLAYER_INVINCIBLE_TIME: f32 = 2.0;

// Enemies
pub const LANDER_SPEED: f32 = 55.0;
pub const LANDER_DETECT_RADIUS: f32 = 160.0;
pub const LANDER_SWOOP_SPEED: f32 = 90.0;
pub const LANDER_ASCEND_SPEED: f32 = 70.0;
pub const MUTANT_SPEED: f32 = 130.0;
pub const BOMBER_SPEED: f32 = 70.0;
pub const BOMBER_BOMB_RATE: f32 = 2.5;
pub const SWARMER_SPEED: f32 = 160.0;
pub const BAITER_SPEED: f32 = 160.0;
pub const BAITER_FIRE_RATE: f32 = 1.5;
pub const BAITER_SPAWN_TIME: f32 = 30.0;
pub const POD_SPEED: f32 = 30.0;
pub const POD_SWARMER_COUNT_MIN: usize = 4;
pub const POD_SWARMER_COUNT_MAX: usize = 8;

pub const LANDER_HALF_W: f32 = 10.0;
pub const LANDER_HALF_H: f32 = 12.0;
pub const ENEMY_TOP_THRESHOLD: f32 = 25.0;

// Bullets
pub const BULLET_HALF_W: f32 = 8.0;
pub const BULLET_HALF_H: f32 = 2.0;
pub const ENEMY_BOMB_SPEED: f32 = 80.0;
pub const ENEMY_BULLET_SPEED: f32 = 200.0;

// Astronauts
pub const ASTRONAUT_COUNT: usize = 10;
pub const ASTRONAUT_FALL_SPEED_INITIAL: f32 = 0.0;
pub const ASTRONAUT_GRAVITY: f32 = 80.0;
pub const ASTRONAUT_CATCH_SCORE_BASE: u32 = 500;
pub const ASTRONAUT_HALF_W: f32 = 5.0;
pub const ASTRONAUT_HALF_H: f32 = 8.0;

// Scoring
pub const SCORE_LANDER: u32 = 150;
pub const SCORE_MUTANT: u32 = 150;
pub const SCORE_BAITER: u32 = 200;
pub const SCORE_BOMBER: u32 = 250;
pub const SCORE_POD: u32 = 1000;
pub const SCORE_SWARMER: u32 = 150;
pub const EXTRA_LIFE_THRESHOLD: u32 = 10000;
pub const MAX_LIVES: u32 = 5;

// Game phases timing
pub const PLAYER_DEAD_DURATION: f32 = 3.0;
pub const LEVEL_COMPLETE_DURATION: f32 = 2.5;

// dt cap
pub const MAX_DT: f32 = 0.05;
