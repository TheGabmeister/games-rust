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
    alive: bool,
}

impl Player {
    fn new(texture: Texture2D) -> Self {
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            speed: 200.0,
            texture,
            alive: true,
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

    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
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
    vy: f32,
    texture: Texture2D,
    alive: bool,
}

impl Laser {
    fn new(x: f32, y: f32, vy: f32, texture: Texture2D) -> Self {
        Self { x, y, vy, texture, alive: true }
    }

    fn update(&mut self, dt: f32) {
        self.y += self.vy * dt;
        let h = self.texture.height();
        if self.y < -h || self.y > screen_height() + h {
            self.alive = false;
        }
    }

    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
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
    alive: bool,
    shoot_timer: f32,
}

impl Enemy {
    fn new(texture: Texture2D) -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            texture,
            alive: true,
            shoot_timer: 5.0,
        }
    }

    // Returns true when it's time to fire.
    fn update(&mut self, dt: f32) -> bool {
        self.shoot_timer -= dt;
        if self.shoot_timer <= 0.0 {
            self.shoot_timer = 5.0;
            return true;
        }
        false
    }

    fn collider(&self) -> Rect {
        let hw = self.texture.width() / 2.0;
        let hh = self.texture.height() / 2.0;
        Rect::new(self.x - hw, self.y - hh, self.texture.width(), self.texture.height())
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
    let mut enemy = Enemy::new(enemy_texture);
    let mut player_lasers: Vec<Laser> = Vec::new();
    let mut enemy_lasers: Vec<Laser> = Vec::new();

    loop {
        let dt = get_frame_time();
        state.update();

        if player.alive {
            player.update(dt);

            if is_key_pressed(KeyCode::Space) {
                player_lasers.push(Laser::new(player.x, player.y, -500.0, laser_texture.clone()));
                play_sound(&sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }
        }

        if enemy.alive {
            if enemy.update(dt) {
                enemy_lasers.push(Laser::new(enemy.x, enemy.y, 500.0, laser_texture.clone()));
                play_sound(&sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }

            for laser in &mut player_lasers {
                if laser.alive && laser.collider().overlaps(&enemy.collider()) {
                    laser.alive = false;
                    enemy.alive = false;
                }
            }
        }

        if player.alive {
            for laser in &mut enemy_lasers {
                if laser.alive && laser.collider().overlaps(&player.collider()) {
                    laser.alive = false;
                    player.alive = false;
                }
            }
        }

        for laser in &mut player_lasers { laser.update(dt); }
        for laser in &mut enemy_lasers  { laser.update(dt); }
        player_lasers.retain(|l| l.alive);
        enemy_lasers.retain(|l| l.alive);

        clear_background(BLACK);
        if player.alive { player.draw(); }
        if enemy.alive  { enemy.draw(); }
        for laser in &player_lasers { laser.draw(); }
        for laser in &enemy_lasers  { laser.draw(); }

        if state.should_quit {
            break;
        }

        next_frame().await
    }
}
