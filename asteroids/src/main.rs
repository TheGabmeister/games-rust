use macroquad::prelude::*;
use serde::Deserialize;

fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[derive(Deserialize)]
struct TextureEntry {
    id: String,
    path: String,
}

#[derive(Deserialize)]
struct Assets {
    textures: Vec<TextureEntry>,
}

impl Assets {
    fn texture_path(&self, id: &str) -> Option<&str> {
        self.textures.iter().find(|t| t.id == id).map(|t| t.path.as_str())
    }
}

struct Player {
    x: f32,
    y: f32,
    speed: f32,
    texture: Texture2D,
}

impl Player {
    fn new(texture: Texture2D) -> Self {
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            speed: 200.0,
            texture,
        }
    }

    fn update(&mut self, dt: f32) {
        let left  = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        let up    = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        let down  = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);

        if left  { self.x -= self.speed * dt; }
        if right { self.x += self.speed * dt; }
        if up    { self.y -= self.speed * dt; }
        if down  { self.y += self.speed * dt; }

        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        self.x = self.x.clamp(hw, screen_width() - hw);
        self.y = self.y.clamp(hh, screen_height() - hh);
    }

    fn draw(&self) {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        draw_texture(&self.texture, self.x - hw, self.y - hh, WHITE);
    }
}

struct GameState {
    should_quit: bool,
}

impl GameState {
    fn new() -> Self {
        Self { should_quit: false }
    }

    fn update(&mut self) {
        if is_key_pressed(KeyCode::Q) {
            self.should_quit = true;
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let json = std::fs::read_to_string("assets/assets.json").expect("Failed to read assets.json");
    let assets: Assets = serde_json::from_str(&json).expect("Failed to parse assets.json");

    let player_path = assets.texture_path("player").expect("No 'player' texture in assets.json");
    let player_texture = load_texture(player_path).await.expect("Failed to load player texture");

    let mut state = GameState::new();
    let mut player = Player::new(player_texture);

    loop {
        let dt = get_frame_time();
        state.update();
        player.update(dt);

        if state.should_quit {
            std::process::exit(0);
        }

        clear_background(BLACK);
        player.draw();

        next_frame().await
    }
}
