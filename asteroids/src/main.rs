#![windows_subsystem = "windows"]
#![cfg_attr(debug_assertions, allow(unused))] // Warn user of unused code during Release builds.

use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;


fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[derive(Deserialize)]
struct AssetEntry {
    path: String,
}

#[derive(Deserialize)]
struct Assets {
    textures: Vec<AssetEntry>,
    sounds: Vec<AssetEntry>,
}

impl Assets {
    fn stem(path: &str) -> String {
        std::path::Path::new(path)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    async fn load_textures(&self) -> HashMap<String, Texture2D> {
        let mut map = HashMap::new();
        for entry in &self.textures {
            let texture = load_texture(&entry.path).await
                .unwrap_or_else(|_| panic!("Failed to load texture: {}", entry.path));
            map.insert(Self::stem(&entry.path), texture);
        }
        map
    }

    async fn load_sounds(&self) -> HashMap<String, Sound> {
        let mut map = HashMap::new();
        for entry in &self.sounds {
            let sound = load_sound(&entry.path).await
                .unwrap_or_else(|_| panic!("Failed to load sound: {}", entry.path));
            map.insert(Self::stem(&entry.path), sound);
        }
        map
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

struct Enemy {
    x: f32,
    y: f32,
    texture: Texture2D,
}

impl Enemy {
    fn new(texture: Texture2D) -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            texture,
        }
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

    let textures = assets.load_textures().await;
    let sounds = assets.load_sounds().await;

    play_sound(sounds.get("music_spaceshooter").expect("No 'music_spaceshooter' sound"), PlaySoundParams { looped: true, volume: 1.0 });

    let mut state = GameState::new();
    let mut player = Player::new(textures["player_ship"].clone());
    let enemy = Enemy::new(textures["enemy_ufo_green"].clone());

    loop {
        let dt = get_frame_time();
        state.update();
        player.update(dt);

        //clear_background(BLACK);
        player.draw();
        enemy.draw();

        if state.should_quit {
            break;
        }

        next_frame().await
    }
}
