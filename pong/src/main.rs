use macroquad::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

const PADDLE_WIDTH: f32 = 12.0;
const PADDLE_HEIGHT: f32 = 80.0;
const BALL_SIZE: f32 = 12.0;
const PADDLE_SPEED: f32 = 400.0;
const INITIAL_BALL_SPEED: f32 = 320.0;
const BALL_SPEED_INCREMENT: f32 = 20.0;
const MAX_BALL_SPEED: f32 = 700.0;
const MAX_BOUNCE_ANGLE: f32 = std::f32::consts::FRAC_PI_3; // 60°
const WINNING_SCORE: u32 = 7;
const PADDLE_MARGIN: f32 = 20.0;
const SCORE_PAUSE_SECS: f32 = 1.5;
const DASH_COUNT: usize = 15;

// ── Player identity ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    One,
    Two,
}

// ── Game phase state machine ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum GamePhase {
    Playing,
    /// Brief pause after a point is scored before resuming.
    Scored { timer: f32, scorer: Player },
    GameOver { winner: Player },
}

// ── Ball ─────────────────────────────────────────────────────────────────────

struct Ball {
    pos: Vec2,
    vel: Vec2,
}

impl Ball {
    fn new() -> Self {
        let serve_to = if rand::gen_range(0u8, 2) == 0 {
            Player::One
        } else {
            Player::Two
        };
        let mut ball = Self {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
        };
        ball.reset_to_center(serve_to);
        ball
    }

    fn reset_to_center(&mut self, serve_towards: Player) {
        self.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
        let dir_x: f32 = match serve_towards {
            Player::One => -1.0,
            Player::Two => 1.0,
        };
        let offset = rand::gen_range(-0.4f32, 0.4f32);
        self.vel = vec2(dir_x, offset).normalize() * INITIAL_BALL_SPEED;
    }

    fn rect(&self) -> Rect {
        let half = BALL_SIZE / 2.0;
        Rect::new(self.pos.x - half, self.pos.y - half, BALL_SIZE, BALL_SIZE)
    }

    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;

        let half = BALL_SIZE / 2.0;
        if self.pos.y - half <= 0.0 {
            self.pos.y = half;
            self.vel.y = self.vel.y.abs();
        } else if self.pos.y + half >= screen_height() {
            self.pos.y = screen_height() - half;
            self.vel.y = -self.vel.y.abs();
        }
    }

    fn draw(&self) {
        let half = BALL_SIZE / 2.0;
        draw_rectangle(self.pos.x - half, self.pos.y - half, BALL_SIZE, BALL_SIZE, WHITE);
    }
}

// ── Paddle ───────────────────────────────────────────────────────────────────

struct Paddle {
    y: f32,
    score: u32,
}

impl Paddle {
    fn new() -> Self {
        Self {
            y: screen_height() / 2.0 - PADDLE_HEIGHT / 2.0,
            score: 0,
        }
    }

    fn rect(&self, x: f32) -> Rect {
        Rect::new(x, self.y, PADDLE_WIDTH, PADDLE_HEIGHT)
    }

    fn update(&mut self, dt: f32, up: bool, down: bool) {
        if up {
            self.y -= PADDLE_SPEED * dt;
        }
        if down {
            self.y += PADDLE_SPEED * dt;
        }
        self.y = self.y.clamp(0.0, screen_height() - PADDLE_HEIGHT);
    }

    fn draw(&self, x: f32) {
        draw_rectangle(x, self.y, PADDLE_WIDTH, PADDLE_HEIGHT, WHITE);
    }
}

// ── Game ─────────────────────────────────────────────────────────────────────

struct Game {
    ball: Ball,
    player1: Paddle,
    player2: Paddle,
    phase: GamePhase,
}

impl Game {
    fn new() -> Self {
        Self {
            ball: Ball::new(),
            player1: Paddle::new(),
            player2: Paddle::new(),
            phase: GamePhase::Playing,
        }
    }

    fn p1_x() -> f32 {
        PADDLE_MARGIN
    }

    fn p2_x() -> f32 {
        screen_width() - PADDLE_MARGIN - PADDLE_WIDTH
    }

    fn update(&mut self, dt: f32) {
        // Paddles remain controllable during play and the post-score pause.
        if matches!(self.phase, GamePhase::Playing | GamePhase::Scored { .. }) {
            self.player1
                .update(dt, is_key_down(KeyCode::W), is_key_down(KeyCode::S));
            self.player2
                .update(dt, is_key_down(KeyCode::Up), is_key_down(KeyCode::Down));
        }

        match self.phase {
            GamePhase::Playing => self.update_ball_and_score(dt),

            GamePhase::Scored { timer, scorer } => {
                let remaining = timer - dt;
                self.phase = if remaining <= 0.0 {
                    GamePhase::Playing
                } else {
                    GamePhase::Scored { timer: remaining, scorer }
                };
            }

            GamePhase::GameOver { .. } => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    *self = Game::new();
                }
            }
        }
    }

    fn update_ball_and_score(&mut self, dt: f32) {
        self.ball.update(dt);

        let p1_x = Self::p1_x();
        let p2_x = Self::p2_x();
        let p1_rect = self.player1.rect(p1_x);
        let p2_rect = self.player2.rect(p2_x);
        let ball_rect = self.ball.rect();

        // Player 1 paddle collision (ball travelling left)
        if ball_rect.overlaps(&p1_rect) && self.ball.vel.x < 0.0 {
            self.ball.pos.x = p1_rect.right() + BALL_SIZE / 2.0;
            let speed = (self.ball.vel.length() + BALL_SPEED_INCREMENT).min(MAX_BALL_SPEED);
            self.ball.vel = bounce_velocity(&self.ball.pos, &p1_rect, speed, true);
        }

        // Player 2 paddle collision (ball travelling right)
        if ball_rect.overlaps(&p2_rect) && self.ball.vel.x > 0.0 {
            self.ball.pos.x = p2_rect.left() - BALL_SIZE / 2.0;
            let speed = (self.ball.vel.length() + BALL_SPEED_INCREMENT).min(MAX_BALL_SPEED);
            self.ball.vel = bounce_velocity(&self.ball.pos, &p2_rect, speed, false);
        }

        // Scoring: ball exits left → P2 scores; exits right → P1 scores.
        let half = BALL_SIZE / 2.0;
        if self.ball.pos.x + half < 0.0 {
            self.award_point(Player::Two);
        } else if self.ball.pos.x - half > screen_width() {
            self.award_point(Player::One);
        }
    }

    fn award_point(&mut self, scorer: Player) {
        let new_score = match scorer {
            Player::One => {
                self.player1.score += 1;
                self.player1.score
            }
            Player::Two => {
                self.player2.score += 1;
                self.player2.score
            }
        };

        if new_score >= WINNING_SCORE {
            self.phase = GamePhase::GameOver { winner: scorer };
        } else {
            // Serve towards the player who just conceded.
            let loser = match scorer {
                Player::One => Player::Two,
                Player::Two => Player::One,
            };
            self.ball.reset_to_center(loser);
            self.phase = GamePhase::Scored { timer: SCORE_PAUSE_SECS, scorer };
        }
    }

    // ── Drawing ──────────────────────────────────────────────────────────────

    fn draw(&self) {
        clear_background(BLACK);
        self.draw_center_line();
        self.draw_scores();
        self.player1.draw(Self::p1_x());
        self.player2.draw(Self::p2_x());
        self.ball.draw();
        self.draw_overlay();
        self.draw_controls_hint();
    }

    fn draw_center_line(&self) {
        let dash_h = screen_height() / (DASH_COUNT as f32 * 2.0 - 1.0);
        let x = screen_width() / 2.0 - 2.0;
        let color = Color::new(0.35, 0.35, 0.35, 1.0);
        for i in 0..DASH_COUNT {
            draw_rectangle(x, i as f32 * dash_h * 2.0, 4.0, dash_h, color);
        }
    }

    fn draw_scores(&self) {
        let cx = screen_width() / 2.0;
        draw_text(&self.player1.score.to_string(), cx - 90.0, 70.0, 60.0, WHITE);
        draw_text(&self.player2.score.to_string(), cx + 50.0, 70.0, 60.0, WHITE);
    }

    fn draw_overlay(&self) {
        match self.phase {
            GamePhase::Scored { scorer, .. } => {
                let msg = match scorer {
                    Player::One => "Player 1 Scores!",
                    Player::Two => "Player 2 Scores!",
                };
                draw_centered(msg, screen_height() / 2.0, 40.0, YELLOW);
            }
            GamePhase::GameOver { winner } => {
                let msg = match winner {
                    Player::One => "Player 1 Wins!",
                    Player::Two => "Player 2 Wins!",
                };
                draw_centered(msg, screen_height() / 2.0 - 20.0, 60.0, YELLOW);
                draw_centered(
                    "Press SPACE or ENTER to play again",
                    screen_height() / 2.0 + 40.0,
                    24.0,
                    WHITE,
                );
            }
            GamePhase::Playing => {}
        }
    }

    fn draw_controls_hint(&self) {
        let y = screen_height() - 10.0;
        let size = 18.0;
        let color = Color::new(0.4, 0.4, 0.4, 1.0);
        draw_text("W / S", 10.0, y, size, color);
        let hint = "\u{2191} / \u{2193}"; // ↑ / ↓
        let tw = measure_text(hint, None, size as u16, 1.0).width;
        draw_text(hint, screen_width() - tw - 10.0, y, size, color);
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Compute the outgoing ball velocity after hitting a paddle.
/// `going_right` is true for Player 1's paddle (ball bounces rightward).
fn bounce_velocity(ball_pos: &Vec2, paddle: &Rect, speed: f32, going_right: bool) -> Vec2 {
    let hit_factor = (ball_pos.y - paddle.center().y) / (PADDLE_HEIGHT / 2.0);
    let angle = hit_factor.clamp(-1.0, 1.0) * MAX_BOUNCE_ANGLE;
    let vx = if going_right { angle.cos() } else { -angle.cos() };
    vec2(vx, angle.sin()) * speed
}

fn draw_centered(text: &str, y: f32, font_size: f32, color: Color) {
    let w = measure_text(text, None, font_size as u16, 1.0).width;
    draw_text(text, screen_width() / 2.0 - w / 2.0, y, font_size, color);
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[macroquad::main("Pong")]
async fn main() {
    let mut game = Game::new();
    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await;
    }
}
