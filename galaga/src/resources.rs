use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::events::GameEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Attract,
    Ready,
    Playing,
    PlayerDeath,
    StageClear,
    GameOver,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageType {
    Normal,
    Challenge,
}

#[derive(Clone, Copy, Debug)]
pub struct GameFlowState {
    pub mode: GameMode,
    pub mode_timer: f32,
    pub mode_before_pause: GameMode,
}

#[derive(Clone, Copy, Debug)]
pub struct StageState {
    pub number: u32,
    pub stage_type: StageType,
    pub spawn_finished: bool,
    pub challenge_hits: u32,
    pub challenge_total: u32,
    pub betrayed_queue: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ScoreState {
    pub score: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct LivesState {
    pub lives: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct HiScore {
    pub value: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct DifficultyState {
    pub dive_interval: f32,
    pub dive_timer: f32,
    pub max_divers: usize,
    pub enemy_fire_interval: f32,
    pub enemy_fire_timer: f32,
    pub enemy_bullet_speed: f32,
    pub dive_speed_multiplier: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct InputState {
    pub move_axis: f32,
    pub fire_pressed: bool,
    pub start_pressed: bool,
    pub pause_pressed: bool,
}

#[derive(Clone, Debug)]
pub enum SpawnCommand {
    PlayerProjectile {
        position: macroquad::prelude::Vec2,
        barrel: u8,
    },
    EnemyProjectile {
        position: macroquad::prelude::Vec2,
        velocity: macroquad::prelude::Vec2,
    },
}

#[derive(Default, Clone, Debug)]
pub struct SpawnQueue {
    pub commands: Vec<SpawnCommand>,
}

#[derive(Clone, Debug)]
pub struct RngState {
    pub rng: SmallRng,
}

#[derive(Clone, Copy, Debug)]
pub struct UiState {
    pub message_timer: f32,
}

#[derive(Default, Clone, Debug)]
pub struct EventQueue {
    pub events: Vec<GameEvent>,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerState {
    pub dual_active: bool,
    pub respawn_timer: f32,
    pub invuln_on_spawn: f32,
}

#[derive(Clone, Debug)]
pub struct Resources {
    pub flow: GameFlowState,
    pub stage: StageState,
    pub score: ScoreState,
    pub lives: LivesState,
    pub hi_score: HiScore,
    pub difficulty: DifficultyState,
    pub input: InputState,
    pub spawn_queue: SpawnQueue,
    pub rng: RngState,
    pub ui: UiState,
    pub events: EventQueue,
    pub player: PlayerState,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            flow: GameFlowState {
                mode: GameMode::Attract,
                mode_timer: 0.0,
                mode_before_pause: GameMode::Playing,
            },
            stage: StageState {
                number: 1,
                stage_type: StageType::Normal,
                spawn_finished: false,
                challenge_hits: 0,
                challenge_total: 0,
                betrayed_queue: 0,
            },
            score: ScoreState { score: 0 },
            lives: LivesState { lives: 3 },
            hi_score: HiScore { value: 0 },
            difficulty: DifficultyState {
                dive_interval: 2.2,
                dive_timer: 1.6,
                max_divers: 1,
                enemy_fire_interval: 1.0,
                enemy_fire_timer: 0.7,
                enemy_bullet_speed: 190.0,
                dive_speed_multiplier: 1.0,
            },
            input: InputState {
                move_axis: 0.0,
                fire_pressed: false,
                start_pressed: false,
                pause_pressed: false,
            },
            spawn_queue: SpawnQueue::default(),
            rng: RngState {
                rng: SmallRng::seed_from_u64(0xC0FFEE),
            },
            ui: UiState { message_timer: 0.0 },
            events: EventQueue::default(),
            player: PlayerState {
                dual_active: false,
                respawn_timer: 0.0,
                invuln_on_spawn: 0.0,
            },
        }
    }
}
