#![windows_subsystem = "windows"]
#![cfg_attr(debug_assertions, allow(unused))] // Warn user of unused code during Release builds.

use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

const ASSETS_DIR: &str = "assets"; // Point this to your primary assets folder

fn window_conf() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

enum TextureAsset {
    PlayerShip,
    PlayerLaser,
    EnemyUfoGreen,
}

impl TextureAsset {
    fn path(&self) -> String {
        let file = match self {
            Self::PlayerShip    => "player_ship.png",
            Self::PlayerLaser   => "player_laser.png",
            Self::EnemyUfoGreen => "enemy_ufo_green.png",
        };
        format!("{}/{}", ASSETS_DIR, file)
    }

    async fn load(&self) -> Texture2D {
        let path = self.path();
        load_texture(&path).await
            .unwrap_or_else(|_| panic!("Failed to load texture: {}", path))
    }
}

enum SoundAsset {
    SfxLaser,
    SfxLose,
    MusicSpaceshooter,
}

impl SoundAsset {
    fn path(&self) -> String {
        let file = match self {
            Self::SfxLaser          => "sfx_laser1.ogg",
            Self::SfxLose           => "sfx_lose.ogg",
            Self::MusicSpaceshooter => "music_spaceshooter.ogg",
        };
        format!("{}/{}", ASSETS_DIR, file)
    }

    async fn load(&self) -> Sound {
        let path = self.path();
        load_sound(&path).await
            .unwrap_or_else(|_| panic!("Failed to load sound: {}", path))
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

struct Laser {
    x: f32,
    y: f32,
    speed: f32,
    texture: Texture2D,
    alive: bool,
}

impl Laser {
    fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self { x, y, speed: 500.0, texture, alive: true }
    }

    fn update(&mut self, dt: f32) {
        self.y -= self.speed * dt;
        if self.y < -self.texture.height() {
            self.alive = false;
        }
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
    let player_texture = TextureAsset::PlayerShip.load().await;
    let laser_texture  = TextureAsset::PlayerLaser.load().await;
    let enemy_texture  = TextureAsset::EnemyUfoGreen.load().await;
    let music          = SoundAsset::MusicSpaceshooter.load().await;
    let sfx_laser      = SoundAsset::SfxLaser.load().await;

    play_sound(&music, PlaySoundParams { looped: true, volume: 1.0 });

    let mut state = GameState::new();
    let mut player = Player::new(player_texture);
    let enemy = Enemy::new(enemy_texture);
    let mut lasers: Vec<Laser> = Vec::new();

    loop {
        let dt = get_frame_time();
        state.update();
        player.update(dt);

        if is_key_pressed(KeyCode::Space) {
            lasers.push(Laser::new(player.x, player.y, laser_texture.clone()));
            play_sound(&sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
        }

        for laser in &mut lasers {
            laser.update(dt);
        }
        lasers.retain(|l| l.alive);

        clear_background(BLACK);
        player.draw();
        enemy.draw();
        for laser in &lasers {
            laser.draw();
        }

        if state.should_quit {
            break;
        }

        next_frame().await
    }
}
