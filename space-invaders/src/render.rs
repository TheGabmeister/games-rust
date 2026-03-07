use crate::game::entities::{BUNKER_COLS, BUNKER_ROWS};
use crate::game::state::{
    DEFEAT_LINE, Game, GameState, PLAYFIELD_PADDING, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use macroquad::prelude::*;

impl Game {
    pub fn draw(&self) {
        clear_background(BLACK);
        self.draw_hud();
        self.draw_defeat_line();
        self.draw_player();
        self.draw_invaders();
        self.draw_bullets();
        self.draw_bunkers();
        self.draw_mystery_ship();
        self.draw_overlay();
    }

    fn draw_hud(&self) {
        let hud_text = format!(
            "Score: {}    Lives: {}    Wave: {}",
            self.score, self.lives, self.wave
        );
        draw_text(&hud_text, 24.0, 30.0, 30.0, GREEN);
    }

    fn draw_defeat_line(&self) {
        draw_line(
            PLAYFIELD_PADDING,
            DEFEAT_LINE,
            SCREEN_WIDTH - PLAYFIELD_PADDING,
            DEFEAT_LINE,
            1.5,
            DARKGRAY,
        );
    }

    fn draw_player(&self) {
        if self.lives <= 0 {
            return;
        }

        draw_rectangle(
            self.player.rect.x,
            self.player.rect.y,
            self.player.rect.w,
            self.player.rect.h,
            WHITE,
        );
        draw_rectangle(
            self.player.rect.x + self.player.rect.w * 0.4,
            self.player.rect.y - 8.0,
            self.player.rect.w * 0.2,
            8.0,
            WHITE,
        );
    }

    fn draw_invaders(&self) {
        for invader in &self.invaders {
            let body_color = match invader.row {
                0 => YELLOW,
                1 | 2 => ORANGE,
                _ => LIME,
            };

            draw_rectangle(
                invader.rect.x,
                invader.rect.y,
                invader.rect.w,
                invader.rect.h,
                body_color,
            );
            draw_rectangle(invader.rect.x + 6.0, invader.rect.y + 6.0, 6.0, 6.0, BLACK);
            draw_rectangle(
                invader.rect.x + invader.rect.w - 12.0,
                invader.rect.y + 6.0,
                6.0,
                6.0,
                BLACK,
            );
        }
    }

    fn draw_bullets(&self) {
        if let Some(bullet) = self.player_bullet {
            draw_rectangle(
                bullet.rect.x,
                bullet.rect.y,
                bullet.rect.w,
                bullet.rect.h,
                WHITE,
            );
        }

        for bullet in &self.invader_bullets {
            draw_rectangle(
                bullet.rect.x,
                bullet.rect.y,
                bullet.rect.w,
                bullet.rect.h,
                RED,
            );
        }
    }

    fn draw_bunkers(&self) {
        for bunker in &self.bunkers {
            for row in 0..BUNKER_ROWS {
                for col in 0..BUNKER_COLS {
                    if bunker.cells[row][col] {
                        draw_rectangle(
                            bunker.position.x + col as f32 * bunker.cell_size,
                            bunker.position.y + row as f32 * bunker.cell_size,
                            bunker.cell_size,
                            bunker.cell_size,
                            GREEN,
                        );
                    }
                }
            }
        }
    }

    fn draw_mystery_ship(&self) {
        if let Some(ship) = self.mystery_ship {
            draw_rectangle(ship.rect.x, ship.rect.y, ship.rect.w, ship.rect.h, RED);
            draw_rectangle(
                ship.rect.x + 10.0,
                ship.rect.y - 6.0,
                ship.rect.w - 20.0,
                6.0,
                RED,
            );
        }
    }

    fn draw_overlay(&self) {
        match self.state {
            GameState::Start => {
                draw_centered_text("SPACE INVADERS", SCREEN_HEIGHT * 0.40, 64.0, WHITE);
                draw_centered_text(
                    "Arrow/A,D to move | Space to fire",
                    SCREEN_HEIGHT * 0.50,
                    30.0,
                    GRAY,
                );
                draw_centered_text("Press Enter to Start", SCREEN_HEIGHT * 0.58, 38.0, GREEN);
            }
            GameState::LifeLost { .. } => {
                draw_centered_text("Life Lost", SCREEN_HEIGHT * 0.50, 52.0, ORANGE);
            }
            GameState::GameOver => {
                draw_centered_text("GAME OVER", SCREEN_HEIGHT * 0.45, 72.0, RED);
                draw_centered_text(
                    &format!("Final Score: {}", self.score),
                    SCREEN_HEIGHT * 0.54,
                    38.0,
                    WHITE,
                );
                draw_centered_text("Press Enter to Restart", SCREEN_HEIGHT * 0.62, 34.0, GREEN);
            }
            GameState::Playing => {}
        }
    }
}

fn draw_centered_text(text: &str, y: f32, font_size: f32, color: Color) {
    let metrics = measure_text(text, None, font_size as u16, 1.0);
    let x = (SCREEN_WIDTH - metrics.width) * 0.5;
    draw_text(text, x, y, font_size, color);
}
