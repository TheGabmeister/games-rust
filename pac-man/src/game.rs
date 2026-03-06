use macroquad::prelude::*;

use crate::entities::{
    fruit_for_level, Direction, Fruit, Ghost, GhostKind, GhostState, LevelConfig, Mover, Pacman,
};
use crate::input::apply_pacman_input;
use crate::maze::{Maze, Pellet};

pub(crate) const MAZE_WIDTH: i32 = 21;
pub(crate) const MAZE_HEIGHT: i32 = 21;
pub(crate) const TILE_SIZE: f32 = 24.0;
pub(crate) const HUD_HEIGHT: f32 = 100.0;

const MAZE_LAYOUT: [&str; MAZE_HEIGHT as usize] = [
    "#####################",
    "#o........#........o#",
    "#.###.###.#.###.###.#",
    "#...................#",
    "#.###.#.#####.#.###.#",
    "#.....#...#...#.....#",
    "#####.###.#.###.#####",
    "    #.#.......#.#    ",
    "#####.#.##-##.#.#####",
    " .......#   #....... ",
    "#####.#.#####.#.#####",
    "    #.#.......#.#    ",
    "#####.#.#####.#.#####",
    "#.........#.........#",
    "#.###.###.#.###.###.#",
    "#o..#...........#..o#",
    "###.#.#.#####.#.#.###",
    "#.....#...#...#.....#",
    "#.#######.#.#######.#",
    "#...................#",
    "#####################",
];

const FRUIT_THRESHOLDS: [usize; 2] = [70, 170];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GameMode {
    Playing,
    LifeLost { timer: f32 },
    Intermission { timer: f32 },
    GameOver,
    KillScreen,
}

pub(crate) struct Game {
    pub(crate) maze: Maze,
    pub(crate) pacman: Pacman,
    pub(crate) ghosts: Vec<Ghost>,
    pub(crate) score: u32,
    pub(crate) high_score: u32,
    pub(crate) level: u16,
    pub(crate) lives: i32,
    pub(crate) mode: GameMode,
    pub(crate) dots_eaten: usize,
    pub(crate) ghost_chain: u8,
    pub(crate) fruit: Option<Fruit>,
    pub(crate) fruit_triggered: [bool; 2],
}

impl Game {
    pub(crate) fn new() -> Self {
        let mut game = Self {
            maze: Maze::from_layout(&MAZE_LAYOUT, 9),
            pacman: Pacman {
                mover: Mover::new(ivec2(10, 15), Direction::Left, 6.0),
                mouth_anim: 0.0,
            },
            ghosts: vec![],
            score: 0,
            high_score: 0,
            level: 1,
            lives: 3,
            mode: GameMode::Playing,
            dots_eaten: 0,
            ghost_chain: 0,
            fruit: None,
            fruit_triggered: [false, false],
        };

        game.reset_positions();
        game
    }

    fn restart(&mut self) {
        let high_score = self.high_score.max(self.score);
        *self = Game::new();
        self.high_score = high_score;
    }

    fn current_level_config(&self) -> LevelConfig {
        LevelConfig::for_level(self.level)
    }

    fn reset_positions(&mut self) {
        let cfg = self.current_level_config();

        self.pacman = Pacman {
            mover: Mover::new(ivec2(10, 15), Direction::Left, cfg.pacman_speed),
            mouth_anim: 0.0,
        };

        self.ghosts = vec![
            Ghost {
                kind: GhostKind::Blinky,
                mover: Mover::new(ivec2(10, 9), Direction::Left, cfg.ghost_speed),
                state: GhostState::Normal,
                home_tile: ivec2(10, 9),
                scatter_target: ivec2(self.maze.width - 2, 1),
                base_color: RED,
                release_delay: 0.0,
            },
            Ghost {
                kind: GhostKind::Pinky,
                mover: Mover::new(ivec2(9, 9), Direction::Up, cfg.ghost_speed),
                state: GhostState::Normal,
                home_tile: ivec2(10, 9),
                scatter_target: ivec2(1, 1),
                base_color: PINK,
                release_delay: 1.0,
            },
            Ghost {
                kind: GhostKind::Inky,
                mover: Mover::new(ivec2(10, 10), Direction::Right, cfg.ghost_speed),
                state: GhostState::Normal,
                home_tile: ivec2(10, 9),
                scatter_target: ivec2(self.maze.width - 2, self.maze.height - 2),
                base_color: SKYBLUE,
                release_delay: 2.5,
            },
            Ghost {
                kind: GhostKind::Clyde,
                mover: Mover::new(ivec2(11, 10), Direction::Left, cfg.ghost_speed),
                state: GhostState::Normal,
                home_tile: ivec2(10, 9),
                scatter_target: ivec2(1, self.maze.height - 2),
                base_color: ORANGE,
                release_delay: 4.0,
            },
        ];

        self.ghost_chain = 0;
    }

    fn start_next_level(&mut self) {
        self.level = self.level.saturating_add(1);

        if self.level >= 256 {
            self.mode = GameMode::KillScreen;
            return;
        }

        self.maze.reset_pellets(&MAZE_LAYOUT);
        self.dots_eaten = 0;
        self.fruit = None;
        self.fruit_triggered = [false, false];
        self.mode = GameMode::Playing;
        self.reset_positions();
    }

    fn lose_life(&mut self) {
        self.lives -= 1;
        self.high_score = self.high_score.max(self.score);
        if self.lives <= 0 {
            self.mode = GameMode::GameOver;
        } else {
            self.mode = GameMode::LifeLost { timer: 1.25 };
        }
    }

    pub(crate) fn update(&mut self, dt: f32) {
        match self.mode {
            GameMode::Playing => self.update_playing(dt),
            GameMode::LifeLost { mut timer } => {
                timer -= dt;
                if timer <= 0.0 {
                    self.mode = GameMode::Playing;
                    self.reset_positions();
                } else {
                    self.mode = GameMode::LifeLost { timer };
                }
            }
            GameMode::Intermission { mut timer } => {
                timer -= dt;
                if timer <= 0.0 {
                    self.start_next_level();
                } else {
                    self.mode = GameMode::Intermission { timer };
                }
            }
            GameMode::GameOver | GameMode::KillScreen => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.restart();
                }
            }
        }
    }

    fn update_playing(&mut self, dt: f32) {
        let cfg = self.current_level_config();

        apply_pacman_input(&mut self.pacman);
        self.pacman.mover.speed = cfg.pacman_speed;
        self.pacman.mouth_anim += dt * 12.0;
        self.pacman.mover.step(&self.maze, dt, true);

        let pac_tile = self.pacman.mover.tile();

        match self.maze.take_pellet(pac_tile) {
            Pellet::Dot => {
                self.score += 10;
                self.dots_eaten += 1;
            }
            Pellet::Power => {
                self.score += 50;
                self.dots_eaten += 1;
                self.apply_power_pellet(cfg.frightened_duration);
            }
            Pellet::None => {}
        }

        self.handle_fruit_spawn();
        self.update_fruit(dt);

        self.update_ghosts(dt, cfg.ghost_speed);

        if self.check_ghost_collisions() {
            return;
        }

        if self.maze.pellets_left() == 0 {
            self.mode = GameMode::Intermission { timer: 3.0 };
            self.high_score = self.high_score.max(self.score);
        }
    }

    fn apply_power_pellet(&mut self, frightened_duration: f32) {
        self.ghost_chain = 0;

        if frightened_duration <= 0.0 {
            return;
        }

        for ghost in &mut self.ghosts {
            match ghost.state {
                GhostState::Eaten => {}
                GhostState::Normal | GhostState::Frightened { .. } => {
                    ghost.state = GhostState::Frightened {
                        timer: frightened_duration,
                    };
                    if ghost.mover.dir != Direction::None {
                        ghost.mover.dir = ghost.mover.dir.opposite();
                    }
                }
            }
        }
    }

    fn handle_fruit_spawn(&mut self) {
        for (idx, threshold) in FRUIT_THRESHOLDS.iter().enumerate() {
            if !self.fruit_triggered[idx] && self.dots_eaten >= *threshold {
                self.fruit_triggered[idx] = true;
                if self.fruit.is_none() {
                    self.fruit = Some(Fruit {
                        kind: fruit_for_level(self.level),
                        tile: ivec2(10, 11),
                        timer: 10.0,
                    });
                }
            }
        }
    }

    fn update_fruit(&mut self, dt: f32) {
        if let Some(fruit) = self.fruit.as_mut() {
            fruit.timer -= dt;
            if fruit.timer <= 0.0 {
                self.fruit = None;
                return;
            }

            if self.pacman.mover.tile() == fruit.tile {
                self.score += fruit.kind.score();
                self.fruit = None;
            }
        }
    }

    fn update_ghosts(&mut self, dt: f32, base_speed: f32) {
        let pac_tile = self.pacman.mover.tile();
        let pac_dir = self.pacman.mover.dir;
        let blinky_tile = self
            .ghosts
            .iter()
            .find(|g| g.kind == GhostKind::Blinky)
            .map(|g| g.mover.tile())
            .unwrap_or(pac_tile);

        for i in 0..self.ghosts.len() {
            let mut speed = base_speed;

            if self.ghosts[i].release_delay > 0.0 {
                self.ghosts[i].release_delay -= dt;
                self.ghosts[i].mover.desired_dir = Direction::Up;
            } else {
                let target = self.ghost_target_tile(i, pac_tile, pac_dir, blinky_tile);
                let frightened = matches!(self.ghosts[i].state, GhostState::Frightened { .. });
                let chosen =
                    choose_ghost_direction(&self.maze, &self.ghosts[i], target, frightened);
                self.ghosts[i].mover.desired_dir = chosen;
            }

            match self.ghosts[i].state {
                GhostState::Normal => {}
                GhostState::Frightened { mut timer } => {
                    timer -= dt;
                    speed *= 0.72;
                    if timer <= 0.0 {
                        self.ghosts[i].state = GhostState::Normal;
                        self.ghost_chain = 0;
                    } else {
                        self.ghosts[i].state = GhostState::Frightened { timer };
                    }
                }
                GhostState::Eaten => {
                    speed *= 1.85;
                    let tile = self.ghosts[i].mover.tile();
                    if tile == self.ghosts[i].home_tile {
                        self.ghosts[i].state = GhostState::Normal;
                        self.ghosts[i].release_delay = 0.5;
                    }
                    self.ghosts[i].mover.desired_dir = choose_ghost_direction(
                        &self.maze,
                        &self.ghosts[i],
                        self.ghosts[i].home_tile,
                        false,
                    );
                }
            }

            if self.maze.is_tunnel_tile(self.ghosts[i].mover.tile()) {
                speed *= 0.64;
            }

            self.ghosts[i].mover.speed = speed;
            self.ghosts[i].mover.step(&self.maze, dt, false);
        }
    }

    fn ghost_target_tile(
        &self,
        idx: usize,
        pac_tile: IVec2,
        pac_dir: Direction,
        blinky_tile: IVec2,
    ) -> IVec2 {
        let ghost = &self.ghosts[idx];

        if matches!(ghost.state, GhostState::Eaten) {
            return ghost.home_tile;
        }

        if matches!(ghost.state, GhostState::Frightened { .. }) {
            return random_maze_tile(&self.maze);
        }

        match ghost.kind {
            GhostKind::Blinky => pac_tile,
            GhostKind::Pinky => pac_tile + pac_dir.vector_i() * 4,
            GhostKind::Inky => {
                let two_ahead = pac_tile + pac_dir.vector_i() * 2;
                two_ahead + (two_ahead - blinky_tile)
            }
            GhostKind::Clyde => {
                let distance = ghost.mover.tile().as_vec2().distance(pac_tile.as_vec2());
                if distance > 8.0 {
                    pac_tile
                } else {
                    ghost.scatter_target
                }
            }
        }
    }

    fn check_ghost_collisions(&mut self) -> bool {
        let mut pacman_hit = false;

        for ghost in &mut self.ghosts {
            if ghost.mover.pos.distance(self.pacman.mover.pos) >= 0.55 {
                continue;
            }

            match ghost.state {
                GhostState::Normal => {
                    pacman_hit = true;
                    break;
                }
                GhostState::Frightened { .. } => {
                    ghost.state = GhostState::Eaten;
                    ghost.mover.desired_dir = Direction::Up;
                    ghost.mover.dir = ghost.mover.dir.opposite();

                    let points = 200 * (1u32 << self.ghost_chain.min(3));
                    self.score += points;
                    self.ghost_chain = (self.ghost_chain + 1).min(4);
                }
                GhostState::Eaten => {}
            }
        }

        if pacman_hit {
            self.lose_life();
            return true;
        }

        false
    }
}

fn can_move(maze: &Maze, tile: IVec2, dir: Direction, is_pacman: bool) -> bool {
    let next_tile = tile + dir.vector_i();
    if is_pacman {
        maze.passable_for_pacman(next_tile)
    } else {
        maze.passable_for_ghost(next_tile)
    }
}

fn choose_ghost_direction(maze: &Maze, ghost: &Ghost, target: IVec2, frightened: bool) -> Direction {
    let tile = ghost.mover.tile();
    let current = ghost.mover.dir;
    let mut candidates: Vec<Direction> = Direction::all()
        .into_iter()
        .filter(|dir| can_move(maze, tile, *dir, false))
        .collect();

    if candidates.is_empty() {
        return current.opposite();
    }

    if candidates.len() > 1 && current != Direction::None {
        candidates.retain(|dir| *dir != current.opposite());
        if candidates.is_empty() {
            candidates.push(current.opposite());
        }
    }

    if frightened {
        let pick = macroquad::rand::gen_range(0, candidates.len() as i32) as usize;
        return candidates[pick];
    }

    let mut best_dir = candidates[0];
    let mut best_dist = f32::MAX;
    for dir in candidates {
        let next_tile = tile + dir.vector_i();
        let dist = next_tile.as_vec2().distance_squared(target.as_vec2());
        if dist < best_dist {
            best_dist = dist;
            best_dir = dir;
        }
    }

    best_dir
}

fn random_maze_tile(maze: &Maze) -> IVec2 {
    loop {
        let x = macroquad::rand::gen_range(1, maze.width - 1);
        let y = macroquad::rand::gen_range(1, maze.height - 1);
        let tile = ivec2(x, y);
        if maze.passable_for_ghost(tile) {
            return tile;
        }
    }
}
