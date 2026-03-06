use macroquad::prelude::*;

const MAZE_WIDTH: i32 = 21;
const MAZE_HEIGHT: i32 = 21;
const TILE_SIZE: f32 = 24.0;
const HUD_HEIGHT: f32 = 100.0;

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
const FRIGHT_FLASH_START: f32 = 1.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    fn vector(self) -> Vec2 {
        match self {
            Direction::Up => vec2(0.0, -1.0),
            Direction::Down => vec2(0.0, 1.0),
            Direction::Left => vec2(-1.0, 0.0),
            Direction::Right => vec2(1.0, 0.0),
            Direction::None => vec2(0.0, 0.0),
        }
    }

    fn vector_i(self) -> IVec2 {
        match self {
            Direction::Up => ivec2(0, -1),
            Direction::Down => ivec2(0, 1),
            Direction::Left => ivec2(-1, 0),
            Direction::Right => ivec2(1, 0),
            Direction::None => ivec2(0, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::None => Direction::None,
        }
    }

    fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ]
    }

    fn angle(self) -> f32 {
        match self {
            Direction::Right => 0.0,
            Direction::Left => std::f32::consts::PI,
            Direction::Up => -std::f32::consts::FRAC_PI_2,
            Direction::Down => std::f32::consts::FRAC_PI_2,
            Direction::None => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Wall,
    Path,
    Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pellet {
    None,
    Dot,
    Power,
}

#[derive(Clone)]
struct Maze {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    pellets: Vec<Pellet>,
    warp_row: i32,
}

impl Maze {
    fn from_layout(layout: &[&str]) -> Self {
        let height = layout.len() as i32;
        let width = layout[0].chars().count() as i32;

        let mut cells = Vec::with_capacity((width * height) as usize);
        let mut pellets = Vec::with_capacity((width * height) as usize);

        for row in layout {
            for ch in row.chars() {
                match ch {
                    '#' => {
                        cells.push(Cell::Wall);
                        pellets.push(Pellet::None);
                    }
                    '.' => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::Dot);
                    }
                    'o' => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::Power);
                    }
                    '-' => {
                        cells.push(Cell::Gate);
                        pellets.push(Pellet::None);
                    }
                    _ => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::None);
                    }
                }
            }
        }

        Self {
            width,
            height,
            cells,
            pellets,
            warp_row: 9,
        }
    }

    fn index(&self, tile: IVec2) -> usize {
        (tile.y * self.width + tile.x) as usize
    }

    fn in_bounds(&self, tile: IVec2) -> bool {
        tile.x >= 0 && tile.x < self.width && tile.y >= 0 && tile.y < self.height
    }

    fn cell(&self, tile: IVec2) -> Option<Cell> {
        if !self.in_bounds(tile) {
            return None;
        }
        Some(self.cells[self.index(tile)])
    }

    fn pellet_at(&self, tile: IVec2) -> Pellet {
        if !self.in_bounds(tile) {
            return Pellet::None;
        }
        self.pellets[self.index(tile)]
    }

    fn take_pellet(&mut self, tile: IVec2) -> Pellet {
        if !self.in_bounds(tile) {
            return Pellet::None;
        }
        let idx = self.index(tile);
        let p = self.pellets[idx];
        self.pellets[idx] = Pellet::None;
        p
    }

    fn pellets_left(&self) -> usize {
        self.pellets
            .iter()
            .filter(|p| matches!(p, Pellet::Dot | Pellet::Power))
            .count()
    }

    fn passable_for_pacman(&self, tile: IVec2) -> bool {
        if !self.in_bounds(tile) {
            return tile.y == self.warp_row;
        }
        !matches!(self.cells[self.index(tile)], Cell::Wall | Cell::Gate)
    }

    fn passable_for_ghost(&self, tile: IVec2) -> bool {
        if !self.in_bounds(tile) {
            return tile.y == self.warp_row;
        }
        !matches!(self.cells[self.index(tile)], Cell::Wall)
    }

    fn reset_pellets(&mut self) {
        *self = Maze::from_layout(&MAZE_LAYOUT);
    }
}
#[derive(Clone, Copy)]
struct Mover {
    pos: Vec2,
    dir: Direction,
    desired_dir: Direction,
    speed: f32,
}

impl Mover {
    fn new(tile: IVec2, dir: Direction, speed: f32) -> Self {
        Self {
            pos: tile_center(tile),
            dir,
            desired_dir: dir,
            speed,
        }
    }

    fn tile(&self) -> IVec2 {
        world_to_tile(self.pos)
    }

    fn is_near_center(&self) -> bool {
        let center = tile_center(self.tile());
        self.pos.distance(center) < 0.08
    }

    fn snap_to_center(&mut self) {
        self.pos = tile_center(self.tile());
    }

    fn step(&mut self, maze: &Maze, dt: f32, is_pacman: bool) {
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
enum GhostKind {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GhostState {
    Normal,
    Frightened { timer: f32 },
    Eaten,
}

#[derive(Clone)]
struct Ghost {
    kind: GhostKind,
    mover: Mover,
    state: GhostState,
    home_tile: IVec2,
    scatter_target: IVec2,
    base_color: Color,
    release_delay: f32,
}

#[derive(Clone)]
struct Pacman {
    mover: Mover,
    mouth_anim: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FruitKind {
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
    fn color(self) -> Color {
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

    fn score(self) -> u32 {
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
struct Fruit {
    kind: FruitKind,
    tile: IVec2,
    timer: f32,
}

#[derive(Clone)]
struct LevelConfig {
    pacman_speed: f32,
    ghost_speed: f32,
    frightened_duration: f32,
}

impl LevelConfig {
    fn for_level(level: u16) -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameMode {
    Playing,
    LifeLost { timer: f32 },
    Intermission { timer: f32 },
    GameOver,
    KillScreen,
}

struct Game {
    maze: Maze,
    pacman: Pacman,
    ghosts: Vec<Ghost>,
    score: u32,
    high_score: u32,
    level: u16,
    lives: i32,
    mode: GameMode,
    dots_eaten: usize,
    ghost_chain: u8,
    fruit: Option<Fruit>,
    fruit_triggered: [bool; 2],
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            maze: Maze::from_layout(&MAZE_LAYOUT),
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

        self.maze.reset_pellets();
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

    fn update(&mut self, dt: f32) {
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

        self.handle_input();
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

    fn handle_input(&mut self) {
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.pacman.mover.desired_dir = Direction::Left;
        } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.pacman.mover.desired_dir = Direction::Right;
        } else if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            self.pacman.mover.desired_dir = Direction::Up;
        } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.pacman.mover.desired_dir = Direction::Down;
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

            if is_in_tunnel(&self.maze, self.ghosts[i].mover.pos) {
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
    fn draw(&self) {
        clear_background(BLACK);
        self.draw_maze();

        match self.mode {
            GameMode::Intermission { timer } => {
                self.draw_intermission(timer);
            }
            GameMode::KillScreen => {
                self.draw_kill_screen();
            }
            _ => {
                if let Some(fruit) = &self.fruit {
                    draw_fruit(fruit, 1.0);
                }

                draw_pacman(&self.pacman);
                for ghost in &self.ghosts {
                    draw_ghost(ghost);
                }
            }
        }

        self.draw_hud();
    }

    fn draw_maze(&self) {
        for y in 0..self.maze.height {
            for x in 0..self.maze.width {
                let tile = ivec2(x, y);
                let pos = tile_to_screen(tile);
                let center = pos + vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);

                match self.maze.cell(tile).unwrap_or(Cell::Wall) {
                    Cell::Wall => {
                        draw_rectangle(
                            pos.x,
                            pos.y,
                            TILE_SIZE,
                            TILE_SIZE,
                            Color::new(0.0, 0.1, 0.45, 1.0),
                        );
                        draw_rectangle_lines(pos.x, pos.y, TILE_SIZE, TILE_SIZE, 1.0, BLUE);
                    }
                    Cell::Gate => {
                        draw_line(
                            pos.x + 2.0,
                            center.y,
                            pos.x + TILE_SIZE - 2.0,
                            center.y,
                            2.0,
                            Color::new(1.0, 0.5, 0.8, 1.0),
                        );
                    }
                    Cell::Path => {}
                }

                match self.maze.pellet_at(tile) {
                    Pellet::Dot => {
                        draw_circle(center.x, center.y, 2.4, Color::new(1.0, 0.9, 0.6, 1.0));
                    }
                    Pellet::Power => {
                        let pulse = ((get_time() as f32 * 7.0).sin() * 0.5 + 0.5) * 2.0;
                        draw_circle(
                            center.x,
                            center.y,
                            4.0 + pulse,
                            Color::new(1.0, 0.9, 0.7, 1.0),
                        );
                    }
                    Pellet::None => {}
                }
            }
        }
    }

    fn draw_hud(&self) {
        let top = MAZE_HEIGHT as f32 * TILE_SIZE;
        draw_rectangle(
            0.0,
            top,
            MAZE_WIDTH as f32 * TILE_SIZE,
            HUD_HEIGHT,
            Color::new(0.02, 0.02, 0.02, 1.0),
        );

        draw_text(
            &format!("SCORE {:06}", self.score),
            14.0,
            top + 28.0,
            28.0,
            WHITE,
        );
        draw_text(
            &format!("HIGH {:06}", self.high_score.max(self.score)),
            14.0,
            top + 56.0,
            24.0,
            LIGHTGRAY,
        );
        draw_text(
            &format!("LEVEL {}", self.level),
            330.0,
            top + 28.0,
            28.0,
            YELLOW,
        );

        for life_idx in 0..self.lives.max(0) {
            let center = vec2(24.0 + life_idx as f32 * 22.0, top + 78.0);
            draw_circle(center.x, center.y, 8.0, YELLOW);
            draw_triangle(
                center,
                center + vec2(8.0, -4.0),
                center + vec2(8.0, 4.0),
                Color::new(0.02, 0.02, 0.02, 1.0),
            );
        }

        let recent_levels = self.level.saturating_sub(6);
        let mut icon_x = MAZE_WIDTH as f32 * TILE_SIZE - 22.0;
        for lvl in (recent_levels..self.level).rev() {
            let kind = fruit_for_level(lvl.max(1));
            draw_circle(icon_x, top + 78.0, 8.0, kind.color());
            icon_x -= 20.0;
        }

        match self.mode {
            GameMode::LifeLost { .. } => {
                draw_text_centered("READY!", top - 16.0, 34.0, YELLOW);
            }
            GameMode::GameOver => {
                draw_text_centered("GAME OVER - Press Enter", top - 16.0, 30.0, RED);
            }
            GameMode::KillScreen => {
                draw_text_centered("KILL SCREEN - Press Enter", top - 16.0, 28.0, ORANGE);
            }
            _ => {}
        }
    }

    fn draw_intermission(&self, timer: f32) {
        let progress = 1.0 - (timer / 3.0).clamp(0.0, 1.0);

        let y = MAZE_HEIGHT as f32 * TILE_SIZE * 0.45;
        draw_line(
            20.0,
            y + 18.0,
            MAZE_WIDTH as f32 * TILE_SIZE - 20.0,
            y + 18.0,
            3.0,
            DARKGRAY,
        );

        let pac_x = -30.0 + progress * (MAZE_WIDTH as f32 * TILE_SIZE + 60.0);
        let blinky_x = MAZE_WIDTH as f32 * TILE_SIZE + 30.0
            - progress * (MAZE_WIDTH as f32 * TILE_SIZE + 120.0);

        draw_circle(pac_x, y, 14.0, YELLOW);
        let mouth = ((progress * 18.0).sin().abs() * 0.45 + 0.1) * std::f32::consts::PI;
        draw_triangle(
            vec2(pac_x, y),
            vec2(pac_x + mouth.cos() * 18.0, y + mouth.sin() * 18.0),
            vec2(pac_x + mouth.cos() * 18.0, y - mouth.sin() * 18.0),
            BLACK,
        );

        let ghost_color = if progress < 0.55 {
            RED
        } else {
            Color::new(0.2, 0.4, 1.0, 1.0)
        };
        draw_ghost_body(vec2(blinky_x, y), 14.0, ghost_color);

        draw_text_centered("INTERMISSION", y - 60.0, 44.0, GOLD);
        draw_text_centered("Pac-Man and Blinky take five.", y - 24.0, 24.0, WHITE);
    }

    fn draw_kill_screen(&self) {
        let split_x = MAZE_WIDTH as f32 * TILE_SIZE * 0.5;

        for y in 0..MAZE_HEIGHT {
            for x in (MAZE_WIDTH / 2)..MAZE_WIDTH {
                let px = x as f32 * TILE_SIZE;
                let py = y as f32 * TILE_SIZE;
                let noise = ((x * 17 + y * 41 + (get_time() as i32 * 11)) % 100) as f32 / 100.0;
                let color = Color::new(0.4 + noise * 0.6, noise * 0.7, 0.2 + noise * 0.7, 1.0);
                draw_rectangle(px, py, TILE_SIZE, TILE_SIZE, color);
            }
        }

        draw_line(
            split_x,
            0.0,
            split_x,
            MAZE_HEIGHT as f32 * TILE_SIZE,
            4.0,
            RED,
        );

        draw_text_centered(
            "LEVEL 256",
            MAZE_HEIGHT as f32 * TILE_SIZE * 0.28,
            58.0,
            YELLOW,
        );
        draw_text_centered(
            "Integer overflow corrupted the maze.",
            MAZE_HEIGHT as f32 * TILE_SIZE * 0.38,
            28.0,
            WHITE,
        );
        draw_text_centered(
            "No valid path remains.",
            MAZE_HEIGHT as f32 * TILE_SIZE * 0.45,
            28.0,
            WHITE,
        );
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

fn choose_ghost_direction(
    maze: &Maze,
    ghost: &Ghost,
    target: IVec2,
    frightened: bool,
) -> Direction {
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

fn draw_pacman(pacman: &Pacman) {
    let center = world_to_screen(pacman.mover.pos);
    let radius = TILE_SIZE * 0.44;
    draw_circle(center.x, center.y, radius, YELLOW);

    let dir = if pacman.mover.dir == Direction::None {
        pacman.mover.desired_dir
    } else {
        pacman.mover.dir
    };

    let mouth_open = (pacman.mouth_anim.sin().abs() * 0.35 + 0.08) * std::f32::consts::PI;
    let angle = dir.angle();
    let p1 = center + vec2((angle + mouth_open).cos(), (angle + mouth_open).sin()) * radius * 1.1;
    let p2 = center + vec2((angle - mouth_open).cos(), (angle - mouth_open).sin()) * radius * 1.1;
    draw_triangle(center, p1, p2, BLACK);
}

fn draw_ghost(ghost: &Ghost) {
    let center = world_to_screen(ghost.mover.pos);
    let radius = TILE_SIZE * 0.44;

    match ghost.state {
        GhostState::Eaten => {
            draw_ghost_eyes(center, radius, ghost.mover.dir, BLUE);
        }
        GhostState::Frightened { timer } => {
            let flashing = timer <= FRIGHT_FLASH_START && ((get_time() * 12.0).sin() > 0.0);
            let body_color = if flashing {
                WHITE
            } else {
                Color::new(0.1, 0.2, 0.95, 1.0)
            };
            draw_ghost_body(center, radius, body_color);

            draw_circle(
                center.x - radius * 0.35,
                center.y - radius * 0.15,
                radius * 0.1,
                WHITE,
            );
            draw_circle(
                center.x + radius * 0.35,
                center.y - radius * 0.15,
                radius * 0.1,
                WHITE,
            );
            draw_line(
                center.x - radius * 0.35,
                center.y + radius * 0.35,
                center.x + radius * 0.35,
                center.y + radius * 0.35,
                2.0,
                WHITE,
            );
        }
        GhostState::Normal => {
            draw_ghost_body(center, radius, ghost.base_color);
            draw_ghost_eyes(center, radius, ghost.mover.dir, BLUE);
        }
    }
}

fn draw_ghost_body(center: Vec2, radius: f32, color: Color) {
    draw_circle(center.x, center.y - radius * 0.15, radius, color);
    draw_rectangle(
        center.x - radius,
        center.y - radius * 0.15,
        radius * 2.0,
        radius * 1.2,
        color,
    );

    let foot_y = center.y + radius * 1.0;
    for i in 0..4 {
        let x = center.x - radius + i as f32 * (radius * 2.0 / 3.0);
        draw_triangle(
            vec2(x, foot_y),
            vec2(x + radius / 3.0, foot_y - radius * 0.25),
            vec2(x + radius * 2.0 / 3.0, foot_y),
            color,
        );
    }
}

fn draw_ghost_eyes(center: Vec2, radius: f32, dir: Direction, pupil_color: Color) {
    let eye_offset = vec2(radius * 0.35, radius * 0.15);
    let pupil_shift = dir.vector() * radius * 0.12;

    let left_eye = center + vec2(-eye_offset.x, -eye_offset.y);
    let right_eye = center + vec2(eye_offset.x, -eye_offset.y);

    draw_circle(left_eye.x, left_eye.y, radius * 0.24, WHITE);
    draw_circle(right_eye.x, right_eye.y, radius * 0.24, WHITE);

    draw_circle(
        left_eye.x + pupil_shift.x,
        left_eye.y + pupil_shift.y,
        radius * 0.1,
        pupil_color,
    );
    draw_circle(
        right_eye.x + pupil_shift.x,
        right_eye.y + pupil_shift.y,
        radius * 0.1,
        pupil_color,
    );
}

fn draw_fruit(fruit: &Fruit, scale: f32) {
    let center = tile_to_screen(fruit.tile) + vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);
    draw_circle(
        center.x,
        center.y,
        TILE_SIZE * 0.26 * scale,
        fruit.kind.color(),
    );
    draw_circle(
        center.x + TILE_SIZE * 0.11 * scale,
        center.y - TILE_SIZE * 0.11 * scale,
        TILE_SIZE * 0.06 * scale,
        WHITE,
    );
}

fn fruit_for_level(level: u16) -> FruitKind {
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

fn tile_to_screen(tile: IVec2) -> Vec2 {
    vec2(tile.x as f32 * TILE_SIZE, tile.y as f32 * TILE_SIZE)
}

fn world_to_screen(world: Vec2) -> Vec2 {
    world * TILE_SIZE
}

fn tile_center(tile: IVec2) -> Vec2 {
    vec2(tile.x as f32 + 0.5, tile.y as f32 + 0.5)
}

fn world_to_tile(world: Vec2) -> IVec2 {
    ivec2(world.x.floor() as i32, world.y.floor() as i32)
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

fn is_in_tunnel(maze: &Maze, pos: Vec2) -> bool {
    let tile = world_to_tile(pos);
    tile.y == maze.warp_row && (tile.x <= 2 || tile.x >= maze.width - 3)
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

fn draw_text_centered(text: &str, y: f32, font_size: f32, color: Color) {
    let m = measure_text(text, None, font_size as u16, 1.0);
    let x = (MAZE_WIDTH as f32 * TILE_SIZE - m.width) * 0.5;
    draw_text(text, x, y, font_size, color);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Pac-Man (Macroquad)".to_string(),
        window_width: (MAZE_WIDTH as f32 * TILE_SIZE) as i32,
        window_height: (MAZE_HEIGHT as f32 * TILE_SIZE + HUD_HEIGHT) as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
