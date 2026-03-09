use macroquad::prelude::*;

use crate::domain::{Enemy, GameConfig, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiOverlayState {
    Title,
    Playing,
    GameOver,
}

pub struct Renderer {
    config: GameConfig,
}

impl Renderer {
    pub fn new(config: GameConfig) -> Self {
        Self { config }
    }

    pub fn draw(&self, world: &World, overlay: UiOverlayState, high_score: u32) {
        clear_background(Color::from_rgba(8, 9, 18, 255));
        let board = self.board_rect();
        let cell_px = self.cell_px(board);

        draw_rectangle(
            board.x - 2.0,
            board.y - 2.0,
            board.w + 4.0,
            board.h + 4.0,
            Color::from_rgba(34, 38, 66, 255),
        );
        draw_rectangle(
            board.x,
            board.y,
            board.w,
            board.h,
            Color::from_rgba(14, 16, 30, 255),
        );

        self.draw_player_zone(board);
        self.draw_mushrooms(world, board, cell_px);
        self.draw_centipedes(world, board, cell_px);
        self.draw_enemies(world, board, cell_px);
        self.draw_projectiles(world, board, cell_px);
        self.draw_player(world, board, cell_px);
        self.draw_hud(world, high_score, overlay);
        self.draw_overlay(world, overlay, board);
    }

    fn board_rect(&self) -> Rect {
        let hud_height = 64.0;
        let logical_w = self.config.logical_width_px();
        let logical_h = self.config.logical_height_px();
        let avail_h = (screen_height() - hud_height).max(1.0);
        let scale = (screen_width() / logical_w).min(avail_h / logical_h);
        let board_w = logical_w * scale;
        let board_h = logical_h * scale;
        let x = (screen_width() - board_w) * 0.5;
        let y = hud_height + (avail_h - board_h) * 0.5;
        Rect::new(x, y, board_w, board_h)
    }

    fn cell_px(&self, board: Rect) -> f32 {
        board.w / self.config.grid_w as f32
    }

    fn cell_origin(&self, board: Rect, cell: Vec2, cell_px: f32) -> Vec2 {
        vec2(board.x + cell.x * cell_px, board.y + cell.y * cell_px)
    }

    fn cell_center(&self, board: Rect, cell: Vec2, cell_px: f32) -> Vec2 {
        self.cell_origin(board, cell, cell_px) + vec2(cell_px * 0.5, cell_px * 0.5)
    }

    fn draw_player_zone(&self, board: Rect) {
        let start_row = self.config.player_area_start_row() as f32;
        let zone_y = board.y + board.h * (start_row / self.config.grid_h as f32);
        draw_rectangle(
            board.x,
            zone_y,
            board.w,
            board.h - (zone_y - board.y),
            Color::from_rgba(20, 28, 36, 180),
        );
    }

    fn draw_mushrooms(&self, world: &World, board: Rect, cell_px: f32) {
        for y in 0..self.config.grid_h {
            for x in 0..self.config.grid_w {
                if let Some(mushroom) = world.mushroom_cell(ivec2(x, y)) {
                    let cell_pos = vec2(x as f32, y as f32);
                    let p = self.cell_origin(board, cell_pos, cell_px);
                    let color = if mushroom.poisoned {
                        Color::from_rgba(172, 66, 184, 255)
                    } else {
                        match mushroom.hp {
                            4 => Color::from_rgba(68, 198, 85, 255),
                            3 => Color::from_rgba(88, 170, 80, 255),
                            2 => Color::from_rgba(115, 145, 79, 255),
                            _ => Color::from_rgba(150, 118, 70, 255),
                        }
                    };
                    draw_rectangle(
                        p.x + cell_px * 0.12,
                        p.y + cell_px * 0.15,
                        cell_px * 0.76,
                        cell_px * 0.72,
                        color,
                    );
                }
            }
        }
    }

    fn draw_centipedes(&self, world: &World, board: Rect, cell_px: f32) {
        for chain in &world.centipede_chains {
            for segment in &chain.segments {
                let center = self.cell_center(board, segment.pos.as_vec2(), cell_px);
                let color = if segment.is_head {
                    Color::from_rgba(59, 226, 84, 255)
                } else {
                    Color::from_rgba(40, 176, 62, 255)
                };
                draw_circle(center.x, center.y, cell_px * 0.37, color);
            }
        }
    }

    fn draw_enemies(&self, world: &World, board: Rect, cell_px: f32) {
        for enemy in &world.enemies {
            match enemy {
                Enemy::Flea(flea) => {
                    let p = self.cell_center(board, flea.pos, cell_px);
                    draw_rectangle(
                        p.x - cell_px * 0.25,
                        p.y - cell_px * 0.32,
                        cell_px * 0.5,
                        cell_px * 0.64,
                        Color::from_rgba(217, 62, 62, 255),
                    );
                }
                Enemy::Spider(spider) => {
                    let p = self.cell_center(board, spider.pos, cell_px);
                    draw_circle(
                        p.x,
                        p.y,
                        cell_px * 0.42,
                        Color::from_rgba(126, 214, 232, 255),
                    );
                    draw_circle(
                        p.x - cell_px * 0.3,
                        p.y,
                        cell_px * 0.18,
                        Color::from_rgba(91, 180, 204, 255),
                    );
                    draw_circle(
                        p.x + cell_px * 0.3,
                        p.y,
                        cell_px * 0.18,
                        Color::from_rgba(91, 180, 204, 255),
                    );
                }
                Enemy::Scorpion(scorpion) => {
                    let p = self.cell_center(board, scorpion.pos, cell_px);
                    draw_rectangle(
                        p.x - cell_px * 0.42,
                        p.y - cell_px * 0.24,
                        cell_px * 0.84,
                        cell_px * 0.48,
                        Color::from_rgba(240, 156, 52, 255),
                    );
                }
                Enemy::DetachedHead(head) => {
                    let p = self.cell_center(board, head.segment.pos.as_vec2(), cell_px);
                    draw_circle(
                        p.x,
                        p.y,
                        cell_px * 0.38,
                        Color::from_rgba(250, 220, 84, 255),
                    );
                }
            }
        }
    }

    fn draw_projectiles(&self, world: &World, board: Rect, cell_px: f32) {
        for projectile in &world.projectiles {
            let p = self.cell_origin(board, projectile.pos, cell_px);
            draw_rectangle(
                p.x + cell_px * 0.43,
                p.y + cell_px * 0.08,
                cell_px * 0.14,
                cell_px * 0.56,
                Color::from_rgba(248, 248, 208, 255),
            );
        }
    }

    fn draw_player(&self, world: &World, board: Rect, cell_px: f32) {
        let p = self.cell_center(board, world.player.pos, cell_px);
        let color = Color::from_rgba(93, 188, 244, 255);
        draw_triangle(
            vec2(p.x, p.y - cell_px * 0.44),
            vec2(p.x - cell_px * 0.4, p.y + cell_px * 0.35),
            vec2(p.x + cell_px * 0.4, p.y + cell_px * 0.35),
            color,
        );
    }

    fn draw_hud(&self, world: &World, high_score: u32, overlay: UiOverlayState) {
        let title = match overlay {
            UiOverlayState::Title => "CENTIPEDE 1981",
            UiOverlayState::Playing => "CENTIPEDE",
            UiOverlayState::GameOver => "GAME OVER",
        };
        draw_text(
            title,
            20.0,
            30.0,
            34.0,
            Color::from_rgba(238, 241, 246, 255),
        );

        let status = format!(
            "SCORE {:06}   HI {:06}   LIVES {}   ROUND {}",
            world.score, high_score, world.lives, world.round
        );
        draw_text(
            &status,
            20.0,
            56.0,
            24.0,
            Color::from_rgba(166, 206, 242, 255),
        );
    }

    fn draw_overlay(&self, _world: &World, overlay: UiOverlayState, board: Rect) {
        match overlay {
            UiOverlayState::Playing => {}
            UiOverlayState::Title => {
                let cx = board.x + board.w * 0.5;
                let cy = board.y + board.h * 0.42;
                draw_text(
                    "PRESS ENTER",
                    cx - 140.0,
                    cy,
                    46.0,
                    Color::from_rgba(245, 245, 160, 255),
                );
                draw_text(
                    "MOVE: WASD / ARROWS  |  FIRE: SPACE",
                    cx - 210.0,
                    cy + 42.0,
                    24.0,
                    Color::from_rgba(190, 210, 230, 255),
                );
            }
            UiOverlayState::GameOver => {
                let cx = board.x + board.w * 0.5;
                let cy = board.y + board.h * 0.45;
                draw_rectangle(
                    board.x + board.w * 0.15,
                    cy - 76.0,
                    board.w * 0.7,
                    128.0,
                    Color::from_rgba(10, 10, 16, 180),
                );
                draw_text(
                    "GAME OVER",
                    cx - 135.0,
                    cy - 20.0,
                    58.0,
                    Color::from_rgba(255, 109, 109, 255),
                );
                draw_text(
                    "PRESS ENTER TO RESTART",
                    cx - 178.0,
                    cy + 26.0,
                    28.0,
                    Color::from_rgba(219, 231, 245, 255),
                );
            }
        }
    }
}
