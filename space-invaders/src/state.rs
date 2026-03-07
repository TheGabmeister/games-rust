use crate::game::entities::{
    BUNKER_COLS, Bullet, Bunker, INVADER_COLS, INVADER_ROWS, Invader, MysteryShip, Player,
};
use macroquad::{prelude::*, rand::gen_range};

pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 720.0;

pub(crate) const PLAYFIELD_PADDING: f32 = 40.0;
pub(crate) const PLAYER_WIDTH: f32 = 48.0;
pub(crate) const PLAYER_HEIGHT: f32 = 24.0;
pub(crate) const PLAYER_Y: f32 = SCREEN_HEIGHT - 62.0;
pub(crate) const PLAYER_SPEED: f32 = 380.0;

pub(crate) const BULLET_WIDTH: f32 = 4.0;
pub(crate) const BULLET_HEIGHT: f32 = 14.0;
pub(crate) const PLAYER_BULLET_SPEED: f32 = -520.0;
pub(crate) const INVADER_BULLET_SPEED: f32 = 260.0;
pub(crate) const MAX_INVADER_BULLETS: usize = 3;

pub(crate) const INVADER_WIDTH: f32 = 34.0;
pub(crate) const INVADER_HEIGHT: f32 = 24.0;
pub(crate) const INVADER_X_GAP: f32 = 16.0;
pub(crate) const INVADER_Y_GAP: f32 = 14.0;
pub(crate) const INVADER_START_X: f32 = 86.0;
pub(crate) const INVADER_BASE_START_Y: f32 = 94.0;
pub(crate) const INVADER_WAVE_Y_STEP: f32 = 12.0;
pub(crate) const INVADER_WAVE_Y_MAX_OFFSET: f32 = 120.0;
pub(crate) const INVADER_STEP_X: f32 = 12.0;
pub(crate) const INVADER_DROP_DISTANCE: f32 = 18.0;

pub(crate) const BUNKER_COUNT: usize = 4;
pub(crate) const BUNKER_CELL_SIZE: f32 = 7.0;
pub(crate) const BUNKER_Y: f32 = SCREEN_HEIGHT - 190.0;

pub(crate) const LIFE_LOST_DELAY: f32 = 1.0;
pub(crate) const DEFEAT_LINE: f32 = PLAYER_Y + 6.0;

pub(crate) const MYSTERY_WIDTH: f32 = 56.0;
pub(crate) const MYSTERY_HEIGHT: f32 = 22.0;
pub(crate) const MYSTERY_Y: f32 = 46.0;
pub(crate) const MYSTERY_SPEED: f32 = 145.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState {
    Start,
    Playing,
    LifeLost { timer: f32 },
    GameOver,
}

pub struct Game {
    pub(crate) state: GameState,
    pub(crate) score: u32,
    pub(crate) lives: i32,
    pub(crate) wave: u32,
    pub(crate) player: Player,
    pub(crate) invaders: Vec<Invader>,
    pub(crate) bunkers: Vec<Bunker>,
    pub(crate) mystery_ship: Option<MysteryShip>,
    pub(crate) player_bullet: Option<Bullet>,
    pub(crate) invader_bullets: Vec<Bullet>,
    pub(crate) swarm_direction: f32,
    pub(crate) swarm_timer: f32,
    pub(crate) invader_shot_timer: f32,
    pub(crate) invader_shot_cooldown: f32,
    pub(crate) mystery_spawn_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        let player_x = (SCREEN_WIDTH - PLAYER_WIDTH) * 0.5;
        let invaders = wave_spawn_layout(1);

        Self {
            state: GameState::Start,
            score: 0,
            lives: 3,
            wave: 1,
            player: Player::new(
                player_x,
                PLAYER_Y,
                PLAYER_WIDTH,
                PLAYER_HEIGHT,
                PLAYER_SPEED,
            ),
            invaders,
            bunkers: spawn_bunkers(),
            mystery_ship: None,
            player_bullet: None,
            invader_bullets: Vec::with_capacity(MAX_INVADER_BULLETS),
            swarm_direction: 1.0,
            swarm_timer: 0.0,
            invader_shot_timer: 0.0,
            invader_shot_cooldown: next_invader_shot_cooldown(INVADER_ROWS * INVADER_COLS),
            mystery_spawn_timer: next_mystery_spawn_delay(),
        }
    }

    pub(crate) fn reset_player_position(&mut self) {
        self.player.rect.x = (SCREEN_WIDTH - PLAYER_WIDTH) * 0.5;
    }
}

pub(crate) fn spawn_bunkers() -> Vec<Bunker> {
    let bunker_width = BUNKER_COLS as f32 * BUNKER_CELL_SIZE;
    let available_width = SCREEN_WIDTH - PLAYFIELD_PADDING * 2.0;
    let total_bunker_width = bunker_width * BUNKER_COUNT as f32;
    let gap = (available_width - total_bunker_width) / (BUNKER_COUNT as f32 - 1.0);

    (0..BUNKER_COUNT)
        .map(|index| {
            let x = PLAYFIELD_PADDING + index as f32 * (bunker_width + gap);
            Bunker::new(vec2(x, BUNKER_Y), BUNKER_CELL_SIZE)
        })
        .collect()
}

pub(crate) fn wave_spawn_layout(wave: u32) -> Vec<Invader> {
    let lowered_offset =
        ((wave.saturating_sub(1)) as f32 * INVADER_WAVE_Y_STEP).min(INVADER_WAVE_Y_MAX_OFFSET);
    let start_y = INVADER_BASE_START_Y + lowered_offset;

    let mut invaders = Vec::with_capacity(INVADER_ROWS * INVADER_COLS);
    for row in 0..INVADER_ROWS {
        for col in 0..INVADER_COLS {
            let x = INVADER_START_X + col as f32 * (INVADER_WIDTH + INVADER_X_GAP);
            let y = start_y + row as f32 * (INVADER_HEIGHT + INVADER_Y_GAP);
            invaders.push(Invader::new(
                x,
                y,
                INVADER_WIDTH,
                INVADER_HEIGHT,
                row,
                col,
                invader_score_for_row(row),
            ));
        }
    }
    invaders
}

pub(crate) fn invader_score_for_row(row: usize) -> u32 {
    match row {
        0 => 30,
        1 | 2 => 20,
        _ => 10,
    }
}

pub(crate) fn next_invader_shot_cooldown(alive_count: usize) -> f32 {
    let pressure = (alive_count.clamp(1, INVADER_ROWS * INVADER_COLS) as f32)
        / (INVADER_ROWS * INVADER_COLS) as f32;
    let base = 0.45 + pressure * 0.85;
    base + gen_range(0.0, 0.35)
}

pub(crate) fn next_mystery_spawn_delay() -> f32 {
    gen_range(10.0, 18.0)
}

pub(crate) fn random_bonus_score() -> u32 {
    match gen_range(0, 4) {
        0 => 50,
        1 => 100,
        2 => 150,
        _ => 300,
    }
}
