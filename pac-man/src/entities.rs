use macroquad::prelude::*;

use crate::maze::Maze;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    pub(crate) fn vector(self) -> Vec2 {
        match self {
            Direction::Up => vec2(0.0, -1.0),
            Direction::Down => vec2(0.0, 1.0),
            Direction::Left => vec2(-1.0, 0.0),
            Direction::Right => vec2(1.0, 0.0),
            Direction::None => vec2(0.0, 0.0),
        }
    }

    pub(crate) fn vector_i(self) -> IVec2 {
        match self {
            Direction::Up => ivec2(0, -1),
            Direction::Down => ivec2(0, 1),
            Direction::Left => ivec2(-1, 0),
            Direction::Right => ivec2(1, 0),
            Direction::None => ivec2(0, 0),
        }
    }

    pub(crate) fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::None => Direction::None,
        }
    }

    pub(crate) fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ]
    }

    pub(crate) fn angle(self) -> f32 {
        match self {
            Direction::Right => 0.0,
            Direction::Left => std::f32::consts::PI,
            Direction::Up => -std::f32::consts::FRAC_PI_2,
            Direction::Down => std::f32::consts::FRAC_PI_2,
            Direction::None => 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Mover {
    pub(crate) pos: Vec2,
    pub(crate) dir: Direction,
    pub(crate) desired_dir: Direction,
    pub(crate) speed: f32,
}

impl Mover {
    pub(crate) fn new(tile: IVec2, dir: Direction, speed: f32) -> Self {
        Self {
            pos: tile_center(tile),
            dir,
            desired_dir: dir,
            speed,
        }
    }

    pub(crate) fn tile(&self) -> IVec2 {
        world_to_tile(self.pos)
    }

    fn is_near_center(&self) -> bool {
        let center = tile_center(self.tile());
        self.pos.distance(center) < 0.08
    }

    fn snap_to_center(&mut self) {
        self.pos = tile_center(self.tile());
    }

    pub(crate) fn step(&mut self, maze: &Maze, dt: f32, is_pacman: bool) {
        if self.is_near_center() {
            self.snap_to_center();

            if self.desired_dir != Direction::None
                && can_move(maze, self.tile(), self.desired_dir, is_pacman)
            {
                self.dir = self.desired_dir;
            }

            if self.dir != Direction::None && !can_move(maze, self.tile(), self.dir, is_pacman) {
                self.dir = Direction::None;
            }
        }

        self.pos += self.dir.vector() * self.speed * dt;
        apply_warp(maze, &mut self.pos);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GhostKind {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GhostState {
    Normal,
    Frightened { timer: f32 },
    Eaten,
}

#[derive(Clone)]
pub(crate) struct Ghost {
    pub(crate) kind: GhostKind,
    pub(crate) mover: Mover,
    pub(crate) state: GhostState,
    pub(crate) home_tile: IVec2,
    pub(crate) scatter_target: IVec2,
    pub(crate) base_color: Color,
    pub(crate) release_delay: f32,
}

#[derive(Clone)]
pub(crate) struct Pacman {
    pub(crate) mover: Mover,
    pub(crate) mouth_anim: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FruitKind {
    Cherry,
    Strawberry,
    Orange,
    Apple,
    Melon,
    Galaxian,
    Bell,
    Key,
}

impl FruitKind {
    pub(crate) fn color(self) -> Color {
        match self {
            FruitKind::Cherry => RED,
            FruitKind::Strawberry => PINK,
            FruitKind::Orange => ORANGE,
            FruitKind::Apple => Color::new(0.0, 0.9, 0.2, 1.0),
            FruitKind::Melon => Color::new(0.1, 0.8, 0.6, 1.0),
            FruitKind::Galaxian => YELLOW,
            FruitKind::Bell => GOLD,
            FruitKind::Key => SKYBLUE,
        }
    }

    pub(crate) fn score(self) -> u32 {
        match self {
            FruitKind::Cherry => 100,
            FruitKind::Strawberry => 300,
            FruitKind::Orange => 500,
            FruitKind::Apple => 700,
            FruitKind::Melon => 1000,
            FruitKind::Galaxian => 2000,
            FruitKind::Bell => 3000,
            FruitKind::Key => 5000,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Fruit {
    pub(crate) kind: FruitKind,
    pub(crate) tile: IVec2,
    pub(crate) timer: f32,
}

#[derive(Clone)]
pub(crate) struct LevelConfig {
    pub(crate) pacman_speed: f32,
    pub(crate) ghost_speed: f32,
    pub(crate) frightened_duration: f32,
}

impl LevelConfig {
    pub(crate) fn for_level(level: u16) -> Self {
        let level_f = level as f32;
        let pacman_speed = (6.0 + level_f * 0.06).min(8.0);
        let ghost_speed = (5.1 + level_f * 0.09).min(8.6);
        let frightened_duration = (6.0 - (level_f - 1.0) * 0.35).max(0.0);

        Self {
            pacman_speed,
            ghost_speed,
            frightened_duration,
        }
    }
}

pub(crate) fn fruit_for_level(level: u16) -> FruitKind {
    match level {
        1 => FruitKind::Cherry,
        2 => FruitKind::Strawberry,
        3 | 4 => FruitKind::Orange,
        5 | 6 => FruitKind::Apple,
        7 | 8 => FruitKind::Melon,
        9 | 10 => FruitKind::Galaxian,
        11 | 12 => FruitKind::Bell,
        _ => FruitKind::Key,
    }
}

pub(crate) fn tile_center(tile: IVec2) -> Vec2 {
    vec2(tile.x as f32 + 0.5, tile.y as f32 + 0.5)
}

pub(crate) fn world_to_tile(world: Vec2) -> IVec2 {
    ivec2(world.x.floor() as i32, world.y.floor() as i32)
}

fn can_move(maze: &Maze, tile: IVec2, dir: Direction, is_pacman: bool) -> bool {
    let next_tile = tile + dir.vector_i();
    if is_pacman {
        maze.passable_for_pacman(next_tile)
    } else {
        maze.passable_for_ghost(next_tile)
    }
}

fn apply_warp(maze: &Maze, pos: &mut Vec2) {
    let tile = world_to_tile(*pos);
    if tile.y != maze.warp_row {
        return;
    }

    if pos.x < -0.5 {
        pos.x = maze.width as f32 - 0.5;
    } else if pos.x > maze.width as f32 + 0.5 {
        pos.x = -0.5;
    }
}
