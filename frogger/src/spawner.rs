use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;

// ── Helper ────────────────────────────────────────────────────────────────────

fn row_y(row: i32) -> f32 {
    OFFSET_Y + row as f32 * TILE
}

fn col_x(col: i32) -> f32 {
    OFFSET_X + col as f32 * TILE
}

// ── Meta entity ───────────────────────────────────────────────────────────────

/// Spawn (or re-spawn) the singleton meta entity with given values.
pub fn spawn_meta(world: &mut World, lives: i32, score: i32, level: i32) -> hecs::Entity {
    world.spawn((
        Lives(lives),
        Score(score),
        Level(level),
        LevelTimer(TIMER_SECS),
        HomesProgress([false; 5]),
    ))
}

/// Find the single meta entity (the one with Lives component).
pub fn find_meta(world: &World) -> Option<Entity> {
    world.query::<(Entity, &Lives)>().iter().next().map(|(e, _)| e)
}

// ── Frog ──────────────────────────────────────────────────────────────────────

pub fn spawn_frog(world: &mut World) -> hecs::Entity {
    let col = FROG_START_COL;
    let row = FROG_START_ROW;
    let x = col_x(col) + (TILE - FROG_W) * 0.5;
    let y = row_y(row) + (TILE - FROG_H) * 0.5;
    world.spawn((
        Frog,
        Position(Vec2::new(x, y)),
        Size(Vec2::new(FROG_W, FROG_H)),
        FrogCell { col, row },
        BestRow(row),
        DrawColor(GREEN),
    ))
}

// ── Homes ─────────────────────────────────────────────────────────────────────

pub fn spawn_homes(world: &mut World) {
    for (idx, &col) in HOME_COLS.iter().enumerate() {
        world.spawn((
            Home { idx, filled: false },
            Position(Vec2::new(col_x(col), row_y(ROW_HOMES))),
            Size(Vec2::new(HOME_WIDTH, TILE)),
            DrawColor(Color::from_rgba(0, 80, 0, 255)),
        ));
    }
}

// ── Fly ───────────────────────────────────────────────────────────────────────

pub fn spawn_fly(world: &mut World) {
    let home_idx = macroquad::rand::gen_range(0, HOME_COUNT);
    let col = HOME_COLS[home_idx];
    let cx = col_x(col) + TILE * 0.5;
    let cy = row_y(ROW_HOMES) + TILE * 0.5;
    world.spawn((
        Fly { home_idx, timer: FLY_LIFETIME_SECS },
        Position(Vec2::new(cx - 6.0, cy - 6.0)),
        Size(Vec2::new(12.0, 12.0)),
        DrawColor(YELLOW),
    ));
}

// ── Road vehicles ─────────────────────────────────────────────────────────────

struct VehicleDef {
    row: i32,
    tile_w: i32,
    count: i32,
    base_speed: f32, // positive = right, negative = left
    color: Color,
}

const ROAD_LANES: &[VehicleDef] = &[
    VehicleDef { row: 7,  tile_w: 1, count: 4, base_speed: -120.0, color: Color::from_rgba(220, 50,  50,  255) },
    VehicleDef { row: 8,  tile_w: 2, count: 2, base_speed:  80.0,  color: Color::from_rgba(50,  100, 200, 255) },
    VehicleDef { row: 9,  tile_w: 1, count: 4, base_speed: -150.0, color: Color::from_rgba(220, 180, 50,  255) },
    VehicleDef { row: 10, tile_w: 3, count: 2, base_speed:  60.0,  color: Color::from_rgba(200, 120, 50,  255) },
    VehicleDef { row: 11, tile_w: 1, count: 4, base_speed:  100.0, color: Color::from_rgba(180, 50,  180, 255) },
];

pub fn spawn_road_lanes(world: &mut World, speed_scale: f32) {
    let game_w = COLS as f32 * TILE;
    for def in ROAD_LANES {
        let entity_w = def.tile_w as f32 * TILE;
        let spacing = game_w / def.count as f32;
        for i in 0..def.count {
            // Stagger initial positions evenly across the game width.
            let x = OFFSET_X + i as f32 * spacing;
            let y = row_y(def.row) + (TILE - entity_w.min(TILE)) * 0.0; // vertically centred in tile
            let ey = row_y(def.row) + (TILE - TILE * 0.85) * 0.5;
            world.spawn((
                Vehicle,
                Position(Vec2::new(x, ey)),
                Size(Vec2::new(entity_w - 4.0, TILE * 0.85)),
                Velocity(def.base_speed * speed_scale),
                DrawColor(def.color),
                WrapBounds { x_min: WRAP_X_MIN, x_max: WRAP_X_MAX },
            ));
            let _ = y; // suppress unused
        }
    }
}

// ── River platforms ───────────────────────────────────────────────────────────

struct PlatformDef {
    row: i32,
    tile_w: i32,
    count: i32,
    base_speed: f32, // positive = right, negative = left
    is_turtle: bool,
    color: Color,
}

const RIVER_LANES: &[PlatformDef] = &[
    PlatformDef { row: 1, tile_w: 2, count: 3, base_speed:  90.0, is_turtle: false, color: Color::from_rgba(139, 90,  43,  255) }, // short logs
    PlatformDef { row: 2, tile_w: 2, count: 3, base_speed: -70.0, is_turtle: true,  color: Color::from_rgba(50,  130, 50,  255) }, // turtles
    PlatformDef { row: 3, tile_w: 3, count: 2, base_speed:  60.0, is_turtle: false, color: Color::from_rgba(120, 75,  30,  255) }, // long logs
    PlatformDef { row: 4, tile_w: 2, count: 3, base_speed: -80.0, is_turtle: false, color: Color::from_rgba(100, 65,  25,  255) }, // medium logs
    PlatformDef { row: 5, tile_w: 2, count: 3, base_speed:  70.0, is_turtle: true,  color: Color::from_rgba(50,  140, 60,  255) }, // turtles
];

pub fn spawn_river_lanes(world: &mut World, speed_scale: f32) {
    let game_w = COLS as f32 * TILE;
    for (lane_idx, def) in RIVER_LANES.iter().enumerate() {
        let entity_w = def.tile_w as f32 * TILE;
        let spacing = game_w / def.count as f32;
        for i in 0..def.count {
            let x = OFFSET_X + i as f32 * spacing;
            let ey = row_y(def.row) + (TILE - TILE * 0.75) * 0.5;
            let eh = TILE * 0.75;

            if def.is_turtle {
                // Stagger dive phase so turtles don't all dive simultaneously.
                let stagger = (lane_idx * 3 + i as usize) as f32 * 2.5 % 8.0;
                let init_timer = 8.0 - stagger;
                world.spawn((
                    Platform,
                    Position(Vec2::new(x, ey)),
                    Size(Vec2::new(entity_w - 4.0, eh)),
                    Velocity(def.base_speed * speed_scale),
                    DrawColor(def.color),
                    WrapBounds { x_min: WRAP_X_MIN, x_max: WRAP_X_MAX },
                    TurtleDive {
                        phase: DivePhase::Surface,
                        timer: init_timer,
                    },
                ));
            } else {
                world.spawn((
                    Platform,
                    Position(Vec2::new(x, ey)),
                    Size(Vec2::new(entity_w - 4.0, eh)),
                    Velocity(def.base_speed * speed_scale),
                    DrawColor(def.color),
                    WrapBounds { x_min: WRAP_X_MIN, x_max: WRAP_X_MAX },
                ));
            }
        }
    }
}

// ── Full world setup ──────────────────────────────────────────────────────────

/// Spawn everything needed at game/level start. `meta_entity` is passed so
/// that the caller can store it; if you call this after world.clear() pass
/// None and a fresh meta entity is created.
pub fn spawn_all(world: &mut World, speed_scale: f32, lives: i32, score: i32, level: i32) {
    spawn_meta(world, lives, score, level);
    spawn_frog(world);
    spawn_homes(world);
    spawn_road_lanes(world, speed_scale);
    spawn_river_lanes(world, speed_scale);
    spawn_fly(world);
}
