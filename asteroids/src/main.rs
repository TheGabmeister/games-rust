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
    PillBlue,
}

impl TextureAsset {
    fn path(&self) -> String {
        let file = match self {
            Self::PlayerShip    => "player_ship.png",
            Self::PlayerLaser   => "player_laser.png",
            Self::EnemyUfoGreen => "enemy_ufo_green.png",
            Self::PillBlue      => "pill_blue.png",
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
    lives: u32,
}

impl Player {
    fn new(texture: Texture2D) -> Self {
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            speed: 200.0,
            texture,
            alive: true,
            lives: 3,
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

struct Pickup {
    x: f32,
    y: f32,
    texture: Texture2D,
    alive: bool,
}

impl Pickup {
    fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self { x, y, texture, alive: true }
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

struct Game {
    player:        Player,
    enemy:         Enemy,
    pickup:        Pickup,
    player_lasers: Vec<Laser>,
    enemy_lasers:  Vec<Laser>,
    laser_texture: Texture2D,
    sfx_laser:     Sound,
    should_quit:   bool,
}

impl Game {
    fn new(
        player_texture: Texture2D,
        laser_texture:  Texture2D,
        enemy_texture:  Texture2D,
        pill_texture:   Texture2D,
        sfx_laser:      Sound,
    ) -> Self {
        Self {
            player:        Player::new(player_texture),
            enemy:         Enemy::new(enemy_texture),
            pickup:        Pickup::new(600.0, 450.0, pill_texture),
            player_lasers: Vec::new(),
            enemy_lasers:  Vec::new(),
            laser_texture,
            sfx_laser,
            should_quit:   false,
        }
    }

    fn update(&mut self, dt: f32) {
        if is_key_pressed(KeyCode::Q) {
            self.should_quit = true;
        }

        if self.player.alive {
            self.player.update(dt);

            if is_key_pressed(KeyCode::Space) {
                self.player_lasers.push(Laser::new(
                    self.player.x, self.player.y, -500.0, self.laser_texture.clone(),
                ));
                play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }
        }

        if self.enemy.alive {
            if self.enemy.update(dt) {
                self.enemy_lasers.push(Laser::new(
                    self.enemy.x, self.enemy.y, 500.0, self.laser_texture.clone(),
                ));
                play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }

            for laser in &mut self.player_lasers {
                if laser.alive && laser.collider().overlaps(&self.enemy.collider()) {
                    laser.alive = false;
                    self.enemy.alive = false;
                }
            }
        }

        if self.player.alive {
            for laser in &mut self.enemy_lasers {
                if laser.alive && laser.collider().overlaps(&self.player.collider()) {
                    laser.alive = false;
                    self.player.alive = false;
                }
            }

            if self.pickup.alive && self.player.collider().overlaps(&self.pickup.collider()) {
                self.pickup.alive = false;
                self.player.lives += 1;
            }
        }

        for laser in &mut self.player_lasers { laser.update(dt); }
        for laser in &mut self.enemy_lasers  { laser.update(dt); }
        self.player_lasers.retain(|l| l.alive);
        self.enemy_lasers.retain(|l| l.alive);
    }

    fn draw(&self) {
        clear_background(BLACK);
        if self.player.alive { self.player.draw(); }
        if self.enemy.alive  { self.enemy.draw(); }
        if self.pickup.alive { self.pickup.draw(); }
        for laser in &self.player_lasers { laser.draw(); }
        for laser in &self.enemy_lasers  { laser.draw(); }

        draw_text(&format!("Lives: {}", self.player.lives), 10.0, 24.0, 24.0, WHITE);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let player_texture = TextureAsset::PlayerShip.load().await;
    let laser_texture  = TextureAsset::PlayerLaser.load().await;
    let enemy_texture  = TextureAsset::EnemyUfoGreen.load().await;
    let pill_texture   = TextureAsset::PillBlue.load().await;
    let music          = SoundAsset::MusicSpaceshooter.load().await;
    let sfx_laser      = SoundAsset::SfxLaser.load().await;

    play_sound(&music, PlaySoundParams { looped: true, volume: 1.0 });

    let mut game = Game::new(player_texture, laser_texture, enemy_texture, pill_texture, sfx_laser);

    loop {
        game.update(get_frame_time());
        game.draw();

        if game.should_quit {
            break;
        }

        next_frame().await
    }
}
