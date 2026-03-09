use macroquad::prelude::*;
use crate::constants::*;
use crate::player::draw_mini_ship;

pub struct Scoring {
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub level: u32,
    next_extra_life: u32,
}

impl Scoring {
    pub fn new() -> Self {
        Scoring {
            score: 0,
            lives: 3,
            high_score: 0,
            level: 1,
            next_extra_life: EXTRA_LIFE_THRESHOLD,
        }
    }

    /// Add points; returns true if an extra life was awarded.
    pub fn add(&mut self, pts: u32) -> bool {
        self.score += pts;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        if self.score >= self.next_extra_life {
            self.next_extra_life += EXTRA_LIFE_THRESHOLD;
            if self.lives < MAX_LIVES {
                self.lives += 1;
                return true;
            }
        }
        false
    }

    /// Lose a life; returns true if game over.
    pub fn lose_life(&mut self) -> bool {
        if self.lives == 0 {
            return true;
        }
        self.lives -= 1;
        self.lives == 0
    }

    pub fn draw_hud(&self, smart_bombs: u32) {
        let sw = screen_width();
        let sh = screen_height();

        // Score — top centre
        let score_str = format!("{:08}", self.score);
        let text_w = measure_text(&score_str, None, 22, 1.0).width;
        draw_text(&score_str, sw / 2.0 - text_w / 2.0, SCANNER_HEIGHT - 3.0, 22.0, WHITE);

        // High score — top centre above score (same area, smaller)
        let hi_str = format!("{:08}", self.high_score);
        let hi_w = measure_text(&hi_str, None, 14, 1.0).width;
        draw_text(&hi_str, sw / 2.0 - hi_w / 2.0, 14.0, 14.0, Color::new(0.6, 0.6, 0.6, 1.0));

        // Level — top right
        let lvl_str = format!("L{}", self.level);
        draw_text(&lvl_str, sw - 50.0, SCANNER_HEIGHT - 3.0, 22.0, YELLOW);

        // Lives — bottom left (mini ship icons)
        let base_y = sh - 10.0;
        for i in 0..self.lives {
            draw_mini_ship(14.0 + i as f32 * 24.0, base_y);
        }

        // Smart bombs — bottom left after lives
        let bomb_x = 14.0 + self.lives as f32 * 24.0 + 12.0;
        for i in 0..smart_bombs {
            let bx = bomb_x + i as f32 * 16.0;
            draw_triangle(
                Vec2::new(bx, base_y - 7.0),
                Vec2::new(bx - 5.0, base_y + 2.0),
                Vec2::new(bx + 5.0, base_y + 2.0),
                Color::new(0.8, 0.3, 1.0, 1.0),
            );
        }
    }
}
