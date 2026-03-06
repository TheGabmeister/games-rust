use macroquad::prelude::*;

use crate::entities::{fruit_for_level, Direction, Fruit, Ghost, GhostState, Pacman};
use crate::game::{Game, GameMode, HUD_HEIGHT, MAZE_HEIGHT, MAZE_WIDTH, TILE_SIZE};
use crate::maze::{Cell, Pellet};

const FRIGHT_FLASH_START: f32 = 1.6;

pub(crate) fn draw(game: &Game) {
    clear_background(BLACK);
    draw_maze(game);

    match game.mode {
        GameMode::Intermission { timer } => {
            draw_intermission(timer);
        }
        GameMode::KillScreen => {
            draw_kill_screen();
        }
        _ => {
            if let Some(fruit) = &game.fruit {
                draw_fruit(fruit, 1.0);
            }

            draw_pacman(&game.pacman);
            for ghost in &game.ghosts {
                draw_ghost(ghost);
            }
        }
    }

    draw_hud(game);
}

fn draw_maze(game: &Game) {
    for y in 0..game.maze.height {
        for x in 0..game.maze.width {
            let tile = ivec2(x, y);
            let pos = tile_to_screen(tile);
            let center = pos + vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);

            match game.maze.cell(tile).unwrap_or(Cell::Wall) {
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

            match game.maze.pellet_at(tile) {
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

fn draw_hud(game: &Game) {
    let top = MAZE_HEIGHT as f32 * TILE_SIZE;
    draw_rectangle(
        0.0,
        top,
        MAZE_WIDTH as f32 * TILE_SIZE,
        HUD_HEIGHT,
        Color::new(0.02, 0.02, 0.02, 1.0),
    );

    draw_text(
        &format!("SCORE {:06}", game.score),
        14.0,
        top + 28.0,
        28.0,
        WHITE,
    );
    draw_text(
        &format!("HIGH {:06}", game.high_score.max(game.score)),
        14.0,
        top + 56.0,
        24.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("LEVEL {}", game.level),
        330.0,
        top + 28.0,
        28.0,
        YELLOW,
    );

    for life_idx in 0..game.lives.max(0) {
        let center = vec2(24.0 + life_idx as f32 * 22.0, top + 78.0);
        draw_circle(center.x, center.y, 8.0, YELLOW);
        draw_triangle(
            center,
            center + vec2(8.0, -4.0),
            center + vec2(8.0, 4.0),
            Color::new(0.02, 0.02, 0.02, 1.0),
        );
    }

    let recent_levels = game.level.saturating_sub(6);
    let mut icon_x = MAZE_WIDTH as f32 * TILE_SIZE - 22.0;
    for lvl in (recent_levels..game.level).rev() {
        let kind = fruit_for_level(lvl.max(1));
        draw_circle(icon_x, top + 78.0, 8.0, kind.color());
        icon_x -= 20.0;
    }

    match game.mode {
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

fn draw_intermission(timer: f32) {
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

fn draw_kill_screen() {
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

    draw_line(split_x, 0.0, split_x, MAZE_HEIGHT as f32 * TILE_SIZE, 4.0, RED);

    draw_text_centered("LEVEL 256", MAZE_HEIGHT as f32 * TILE_SIZE * 0.28, 58.0, YELLOW);
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

fn draw_text_centered(text: &str, y: f32, font_size: f32, color: Color) {
    let m = measure_text(text, None, font_size as u16, 1.0);
    let x = (MAZE_WIDTH as f32 * TILE_SIZE - m.width) * 0.5;
    draw_text(text, x, y, font_size, color);
}

fn tile_to_screen(tile: IVec2) -> Vec2 {
    vec2(tile.x as f32 * TILE_SIZE, tile.y as f32 * TILE_SIZE)
}

fn world_to_screen(world: Vec2) -> Vec2 {
    world * TILE_SIZE
}

pub(crate) fn window_conf() -> Conf {
    Conf {
        window_title: "Pac-Man (Macroquad)".to_string(),
        window_width: (MAZE_WIDTH as f32 * TILE_SIZE) as i32,
        window_height: (MAZE_HEIGHT as f32 * TILE_SIZE + HUD_HEIGHT) as i32,
        window_resizable: false,
        ..Default::default()
    }
}
