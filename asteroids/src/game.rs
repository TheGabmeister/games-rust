use macroquad::prelude::*;

struct Player {
    pos: Vec2,
    size: Vec2,
    speed: f32,
}

pub struct Game {
    pub should_quit: bool,
    player: Player,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            should_quit: false,
            player: Player {
                pos: vec2(380.0, 280.0),
                size: vec2(40.0, 40.0),
                speed: 260.0,
            },
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.should_quit = true;
        }

        let mut direction = vec2(0.0, 0.0);
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            direction.x += 1.0;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            direction.y += 1.0;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        self.player.pos += direction * self.player.speed * get_frame_time();

        self.player.pos.x = self
            .player
            .pos
            .x
            .clamp(0.0, screen_width() - self.player.size.x);
        self.player.pos.y = self
            .player
            .pos
            .y
            .clamp(0.0, screen_height() - self.player.size.y);
    }

    pub fn draw(&self) {
        clear_background(BLACK);
        draw_text("Asteroids", 20.0, 40.0, 40.0, WHITE);
        draw_text("Move: WASD or Arrow Keys", 20.0, 70.0, 24.0, GRAY);
        draw_text("Quit: ESC", 20.0, 96.0, 24.0, GRAY);

        draw_rectangle(
            self.player.pos.x,
            self.player.pos.y,
            self.player.size.x,
            self.player.size.y,
            GREEN,
        );
    }
}
