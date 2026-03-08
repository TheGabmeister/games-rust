use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

struct Player {
    x: f32,
    y: f32,
    speed: f32,
    radius: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            speed: 200.0,
            radius: 15.0,
        }
    }

    fn update(&mut self, dt: f32) {
        let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        let down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);

        if left  { self.x -= self.speed * dt; }
        if right { self.x += self.speed * dt; }
        if up    { self.y -= self.speed * dt; }
        if down  { self.y += self.speed * dt; }

        self.x = self.x.clamp(self.radius, screen_width() - self.radius);
        self.y = self.y.clamp(self.radius, screen_height() - self.radius);
    }

    fn draw(&self) {
        draw_circle(self.x, self.y, self.radius, WHITE);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player::new();

    loop {
        let dt = get_frame_time();

        player.update(dt);

        clear_background(BLACK);
        player.draw();

        next_frame().await
    }
}
