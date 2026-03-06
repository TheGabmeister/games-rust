use macroquad::prelude::*;

pub const INVADER_ROWS: usize = 5;
pub const INVADER_COLS: usize = 11;
pub const BUNKER_ROWS: usize = 6;
pub const BUNKER_COLS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulletOwner {
    Player,
    Invader,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub rect: Rect,
    pub speed: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            speed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Invader {
    pub rect: Rect,
    pub row: usize,
    pub col: usize,
    pub score_value: u32,
}

impl Invader {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        row: usize,
        col: usize,
        score_value: u32,
    ) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            row,
            col,
            score_value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bullet {
    pub rect: Rect,
    pub velocity: Vec2,
    pub owner: BulletOwner,
}

impl Bullet {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        velocity: Vec2,
        owner: BulletOwner,
    ) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            velocity,
            owner,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bunker {
    pub position: Vec2,
    pub cell_size: f32,
    pub cells: [[bool; BUNKER_COLS]; BUNKER_ROWS],
}

impl Bunker {
    pub fn new(position: Vec2, cell_size: f32) -> Self {
        Self {
            position,
            cell_size,
            cells: default_bunker_cells(),
        }
    }

    pub fn width(&self) -> f32 {
        BUNKER_COLS as f32 * self.cell_size
    }

    pub fn height(&self) -> f32 {
        BUNKER_ROWS as f32 * self.cell_size
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.width(),
            self.height(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MysteryShip {
    pub rect: Rect,
    pub direction: f32,
    pub speed: f32,
    pub score_value: u32,
}

impl MysteryShip {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        direction: f32,
        speed: f32,
        score_value: u32,
    ) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            direction,
            speed,
            score_value,
        }
    }
}

fn default_bunker_cells() -> [[bool; BUNKER_COLS]; BUNKER_ROWS] {
    [
        [false, true, true, true, true, true, true, true, true, false],
        [true, true, true, true, true, true, true, true, true, true],
        [true, true, true, true, true, true, true, true, true, true],
        [
            true, true, true, false, false, false, false, true, true, true,
        ],
        [
            true, true, false, false, false, false, false, false, true, true,
        ],
        [
            true, false, false, false, false, false, false, false, false, true,
        ],
    ]
}
