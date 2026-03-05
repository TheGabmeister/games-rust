use macroquad::prelude::*;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const TOP_MARGIN: f32 = 70.0;
const SIDE_MARGIN: f32 = 24.0;
const BRICK_COLUMNS: usize = 14;
const BRICK_ROWS: usize = 8;
const BRICK_HEIGHT: f32 = 18.0;
const BRICK_GAP: f32 = 4.0;
const PADDLE_WIDTH: f32 = 110.0;
const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_SPEED: f32 = 520.0;
const BALL_RADIUS: f32 = 8.0;
const BALL_SPEED: f32 = 360.0;
const START_LIVES: i32 = 3;
const MAX_FRAME_DT: f32 = 1.0 / 30.0;
const MIN_BALL_STEP: f32 = BALL_RADIUS * 0.75;

fn window_conf() -> Conf {
    Conf {
        window_title: "Breakout (1976 Style)".to_string(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        sample_count: 1,
        ..Default::default()
    }
}

#[derive(Clone)]
struct Brick {
    rect: Rect,
    alive: bool,
    color: Color,
    value: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Serve,
    Playing,
    Won,
    Lost,
}

struct GameState {
    paddle: Rect,
    ball_pos: Vec2,
    ball_vel: Vec2,
    bricks: Vec<Brick>,
    remaining_bricks: usize,
    score: u32,
    lives: i32,
    phase: Phase,
}

impl GameState {
    fn new() -> Self {
        let paddle = Rect::new(
            (SCREEN_WIDTH - PADDLE_WIDTH) * 0.5,
            SCREEN_HEIGHT - 48.0,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
        );

        let bricks = build_bricks();
        let mut state = Self {
            paddle,
            ball_pos: vec2(0.0, 0.0),
            ball_vel: vec2(0.0, 0.0),
            remaining_bricks: bricks.len(),
            bricks,
            score: 0,
            lives: START_LIVES,
            phase: Phase::Serve,
        };
        state.reset_ball_on_paddle();
        state
    }

    fn restart(&mut self) {
        self.paddle.x = (SCREEN_WIDTH - PADDLE_WIDTH) * 0.5;
        self.bricks = build_bricks();
        self.remaining_bricks = self.bricks.len();
        self.score = 0;
        self.lives = START_LIVES;
        self.phase = Phase::Serve;
        self.reset_ball_on_paddle();
    }

    fn reset_ball_on_paddle(&mut self) {
        self.ball_pos = vec2(
            self.paddle.x + self.paddle.w * 0.5,
            self.paddle.y - BALL_RADIUS - 1.0,
        );
        self.ball_vel = vec2(0.0, 0.0);
    }

    fn is_game_over(&self) -> bool {
        matches!(self.phase, Phase::Won | Phase::Lost)
    }

    fn update(&mut self, dt: f32) {
        self.update_paddle(dt);

        match self.phase {
            Phase::Serve => {
                self.reset_ball_on_paddle();
                if is_key_pressed(KeyCode::Space) {
                    self.phase = Phase::Playing;
                    self.ball_vel = vec2(BALL_SPEED * 0.8, -BALL_SPEED).normalize() * BALL_SPEED;
                }
            }
            Phase::Playing => self.update_ball(dt),
            Phase::Won | Phase::Lost => {}
        }
    }

    fn update_paddle(&mut self, dt: f32) {
        let mut dir = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dir -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dir += 1.0;
        }

        self.paddle.x += dir * PADDLE_SPEED * dt;
        self.paddle.x = self.paddle.x.clamp(0.0, SCREEN_WIDTH - self.paddle.w);
    }

    fn update_ball(&mut self, dt: f32) {
        let steps = ((self.ball_vel.length() * dt) / MIN_BALL_STEP)
            .ceil()
            .max(1.0) as usize;
        let step_dt = dt / steps as f32;

        for _ in 0..steps {
            let previous = self.ball_pos;
            self.ball_pos += self.ball_vel * step_dt;

            self.handle_wall_collision();
            self.handle_paddle_collision();

            if self.handle_brick_collision(previous) && self.remaining_bricks == 0 {
                self.phase = Phase::Won;
                return;
            }

            if self.ball_pos.y > SCREEN_HEIGHT + BALL_RADIUS {
                self.lives -= 1;
                if self.lives <= 0 {
                    self.phase = Phase::Lost;
                } else {
                    self.phase = Phase::Serve;
                    self.reset_ball_on_paddle();
                }
                return;
            }
        }
    }

    fn handle_wall_collision(&mut self) {
        if self.ball_pos.x <= BALL_RADIUS {
            self.ball_pos.x = BALL_RADIUS;
            self.ball_vel.x = self.ball_vel.x.abs();
        }
        if self.ball_pos.x >= SCREEN_WIDTH - BALL_RADIUS {
            self.ball_pos.x = SCREEN_WIDTH - BALL_RADIUS;
            self.ball_vel.x = -self.ball_vel.x.abs();
        }
        if self.ball_pos.y <= BALL_RADIUS {
            self.ball_pos.y = BALL_RADIUS;
            self.ball_vel.y = self.ball_vel.y.abs();
        }
    }

    fn handle_paddle_collision(&mut self) {
        if circle_rect_intersect(self.ball_pos, BALL_RADIUS, self.paddle) && self.ball_vel.y > 0.0 {
            let hit = ((self.ball_pos.x - self.paddle.x) / self.paddle.w).clamp(0.0, 1.0);
            let angle = (-75.0 + hit * 150.0).to_radians();
            self.ball_vel = vec2(angle.sin(), -angle.cos()).normalize() * BALL_SPEED;
            self.ball_pos.y = self.paddle.y - BALL_RADIUS - 0.5;
        }
    }

    fn handle_brick_collision(&mut self, previous_pos: Vec2) -> bool {
        let mut hit_index = None;
        for (idx, brick) in self.bricks.iter().enumerate() {
            if brick.alive && circle_rect_intersect(self.ball_pos, BALL_RADIUS, brick.rect) {
                hit_index = Some(idx);
                break;
            }
        }

        if let Some(idx) = hit_index {
            let brick = &mut self.bricks[idx];
            brick.alive = false;
            self.score += brick.value;
            self.remaining_bricks -= 1;

            let rect = brick.rect;
            let from_above = previous_pos.y + BALL_RADIUS <= rect.y;
            let from_below = previous_pos.y - BALL_RADIUS >= rect.y + rect.h;
            let from_left = previous_pos.x + BALL_RADIUS <= rect.x;
            let from_right = previous_pos.x - BALL_RADIUS >= rect.x + rect.w;

            if from_above {
                self.ball_pos.y = rect.y - BALL_RADIUS;
                self.ball_vel.y = -self.ball_vel.y.abs();
            } else if from_below {
                self.ball_pos.y = rect.y + rect.h + BALL_RADIUS;
                self.ball_vel.y = self.ball_vel.y.abs();
            } else if from_left {
                self.ball_pos.x = rect.x - BALL_RADIUS;
                self.ball_vel.x = -self.ball_vel.x.abs();
            } else if from_right {
                self.ball_pos.x = rect.x + rect.w + BALL_RADIUS;
                self.ball_vel.x = self.ball_vel.x.abs();
            } else {
                self.ball_vel.y = -self.ball_vel.y;
            }

            return true;
        }

        false
    }

    fn draw(&self) {
        clear_background(BLACK);

        for brick in &self.bricks {
            if brick.alive {
                draw_rectangle(
                    brick.rect.x,
                    brick.rect.y,
                    brick.rect.w,
                    brick.rect.h,
                    brick.color,
                );
            }
        }

        draw_rectangle(
            self.paddle.x,
            self.paddle.y,
            self.paddle.w,
            self.paddle.h,
            LIGHTGRAY,
        );
        draw_circle(self.ball_pos.x, self.ball_pos.y, BALL_RADIUS, WHITE);

        draw_hud(self);

        match self.phase {
            Phase::Serve => draw_center_message(&["PRESS SPACE TO LAUNCH"]),
            Phase::Won => draw_center_message(&["YOU WIN", "PRESS R TO RESTART"]),
            Phase::Lost => draw_center_message(&["GAME OVER", "PRESS R TO RESTART"]),
            Phase::Playing => {}
        }
    }
}

fn build_bricks() -> Vec<Brick> {
    let brick_width = (SCREEN_WIDTH - SIDE_MARGIN * 2.0 - BRICK_GAP * (BRICK_COLUMNS as f32 - 1.0))
        / BRICK_COLUMNS as f32;

    let row_styles: [(Color, u32); BRICK_ROWS] = [
        (RED, 7),
        (RED, 7),
        (ORANGE, 5),
        (ORANGE, 5),
        (GREEN, 3),
        (GREEN, 3),
        (YELLOW, 1),
        (YELLOW, 1),
    ];

    let mut bricks = Vec::with_capacity(BRICK_ROWS * BRICK_COLUMNS);
    for (row, (color, value)) in row_styles.iter().enumerate() {
        let y = TOP_MARGIN + row as f32 * (BRICK_HEIGHT + BRICK_GAP);
        for col in 0..BRICK_COLUMNS {
            let x = SIDE_MARGIN + col as f32 * (brick_width + BRICK_GAP);
            bricks.push(Brick {
                rect: Rect::new(x, y, brick_width, BRICK_HEIGHT),
                alive: true,
                color: *color,
                value: *value,
            });
        }
    }
    bricks
}

fn circle_rect_intersect(center: Vec2, radius: f32, rect: Rect) -> bool {
    let nearest_x = center.x.clamp(rect.x, rect.x + rect.w);
    let nearest_y = center.y.clamp(rect.y, rect.y + rect.h);
    let dx = center.x - nearest_x;
    let dy = center.y - nearest_y;
    dx * dx + dy * dy <= radius * radius
}

fn draw_hud(state: &GameState) {
    let score_text = format!("SCORE {:04}", state.score);
    let lives_text = format!("LIVES {}", state.lives.max(0));

    draw_text(&score_text, 20.0, 32.0, 32.0, WHITE);
    let lives_w = measure_text(&lives_text, None, 32, 1.0).width;
    draw_text(
        &lives_text,
        SCREEN_WIDTH - lives_w - 20.0,
        32.0,
        32.0,
        WHITE,
    );
}

fn draw_center_message(lines: &[&str]) {
    let mut y = SCREEN_HEIGHT * 0.5 - (lines.len() as f32 * 18.0);
    for line in lines {
        let dims = measure_text(line, None, 36, 1.0);
        draw_text(line, (SCREEN_WIDTH - dims.width) * 0.5, y, 36.0, WHITE);
        y += 44.0;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();

    loop {
        if is_key_pressed(KeyCode::R) {
            state.restart();
        }

        let dt = get_frame_time().min(MAX_FRAME_DT);
        if !state.is_game_over() {
            state.update(dt);
        }

        state.draw();
        next_frame().await;
    }
}
