use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

use crate::assets::Assets;
use crate::asteroid::Asteroid;
use crate::collidable::{draw_debug, overlaps};
use crate::enemy::Enemy;
use crate::input::InputState;
use crate::laser::Laser;
use crate::particles::ParticleSystem;
use crate::pickup::Pickup;
use crate::player::Player;
use crate::screen_shake::ScreenShake;
use crate::starfield::Starfield;

const MAX_FRAME_DT: f32 = 1.0 / 30.0;

pub struct Game {
    player:        Player,
    enemy:         Enemy,
    pickup:        Pickup,
    asteroids:     Vec<Asteroid>,
    player_lasers: Vec<Laser>,
    enemy_lasers:  Vec<Laser>,
    laser_texture:       Texture2D,
    enemy_laser_texture: Texture2D,
    sfx_laser:     Sound,
    sfx_bump:      Sound,
    sfx_lose:      Sound,
    pub should_quit: bool,
    score:         u32,
    respawn_timer: Option<f32>,
    debug:         bool,
    particles:     ParticleSystem,
    shake:         ScreenShake,
    starfield:     Starfield,
}

impl Game {
    pub fn new(assets: &Assets) -> Self {
        Self {
            player:        Player::new(assets.player_ship.clone()),
            enemy:         Enemy::new(assets.enemy_ufo_green.clone()),
            pickup:        Pickup::new(600.0, 450.0, assets.pill_blue.clone()),
            asteroids:     vec![
                Asteroid::new(100.0, 100.0, assets.asteroid_big.clone()),
                Asteroid::new(700.0, 100.0, assets.asteroid_big.clone()),
                Asteroid::new(100.0, 500.0, assets.asteroid_big.clone()),
                Asteroid::new(700.0, 500.0, assets.asteroid_big.clone()),
            ],
            player_lasers: Vec::new(),
            enemy_lasers:  Vec::new(),
            laser_texture:       assets.player_laser.clone(),
            enemy_laser_texture: assets.enemy_laser.clone(),
            sfx_laser:     assets.sfx_laser.clone(),
            sfx_bump:      assets.sfx_bump.clone(),
            sfx_lose:      assets.sfx_lose.clone(),
            should_quit:   false,
            score:         0,
            respawn_timer: None,
            debug:         false,
            particles:     ParticleSystem::new(),
            shake:         ScreenShake::new(),
            starfield:     Starfield::new(800.0, 600.0, 120),
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        let dt = dt.clamp(0.0, MAX_FRAME_DT);
        self.starfield.update(dt);

        if input.quit {
            self.should_quit = true;
        }

        if is_key_pressed(KeyCode::F1) {
            self.debug = !self.debug;
        }

        if self.player.alive {
            self.player.update(dt, input);

            if input.move_up {
                self.particles.spawn_thrust(
                    self.player.transform.x,
                    self.player.transform.y,
                    self.player.transform.rot,
                    2,
                );
            }

            if input.shoot {
                let speed = 500.0_f32;
                let rot = self.player.transform.rot;
                self.player_lasers.push(Laser::new(
                    self.player.transform.x, self.player.transform.y,
                    rot.sin() * speed,
                    -rot.cos() * speed,
                    self.laser_texture.clone(),
                ));
                play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }
        }

        if self.enemy.alive && self.enemy.update(dt) {
            self.enemy_lasers.push(Laser::new(
                self.enemy.transform.x, self.enemy.transform.y,
                0.0, 500.0,
                self.enemy_laser_texture.clone(),
            ));
            play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
        }

        for asteroid in &mut self.asteroids { asteroid.update(dt); }
        for laser in &mut self.player_lasers { laser.update(dt); }
        for laser in &mut self.enemy_lasers  { laser.update(dt); }

        // Enemy vs player laser
        if self.enemy.alive {
            for laser in &mut self.player_lasers {
                if laser.alive && overlaps(laser, &self.enemy) {
                    laser.alive = false;
                    self.enemy.alive = false;
                    self.particles.spawn_burst(
                        self.enemy.transform.x, self.enemy.transform.y,
                        20, Color::new(0.0, 1.0, 0.7, 1.0), 200.0, 0.6, 4.0,
                    );
                    self.shake.add_trauma(0.2);
                }
            }
        }

        // Asteroid vs player laser
        for asteroid in &mut self.asteroids {
            for laser in &mut self.player_lasers {
                if laser.alive && asteroid.alive && overlaps(laser, asteroid) {
                    laser.alive = false;
                    self.particles.spawn_burst(
                        asteroid.transform.x, asteroid.transform.y,
                        15, Color::new(0.8, 0.5, 0.2, 1.0), 180.0, 0.5, 3.5,
                    );
                    self.shake.add_trauma(0.15);
                    asteroid.alive = false;
                    self.score += 100;
                    play_sound(&self.sfx_bump, PlaySoundParams { looped: false, volume: 1.0 });
                }
            }
        }

        // Player death collisions
        let mut player_killed = false;
        if self.player.alive {
            for laser in &mut self.enemy_lasers {
                if laser.alive && overlaps(laser, &self.player) {
                    laser.alive = false;
                    player_killed = true;
                }
            }

            for asteroid in &self.asteroids {
                if asteroid.alive && overlaps(&self.player, asteroid) {
                    player_killed = true;
                    break;
                }
            }

            if self.enemy.alive && overlaps(&self.player, &self.enemy) {
                self.particles.spawn_burst(
                    self.enemy.transform.x, self.enemy.transform.y,
                    20, Color::new(0.0, 1.0, 0.7, 1.0), 200.0, 0.6, 4.0,
                );
                self.shake.add_trauma(0.2);
                self.enemy.alive = false;
                player_killed = true;
            }

            if self.pickup.alive && overlaps(&self.player, &self.pickup) {
                self.pickup.alive = false;
                self.player.lives += 1;
            }
        }

        if player_killed {
            self.particles.spawn_burst(
                self.player.transform.x, self.player.transform.y,
                25, Color::new(1.0, 1.0, 0.6, 1.0), 250.0, 0.8, 5.0,
            );
            self.shake.add_trauma(0.5);
            self.player.alive = false;
            self.player.lives = self.player.lives.saturating_sub(1);
            play_sound(&self.sfx_lose, PlaySoundParams { looped: false, volume: 1.0 });
            if self.player.lives > 0 {
                self.respawn_timer = Some(3.0);
            }
        }

        if let Some(ref mut timer) = self.respawn_timer {
            *timer -= dt;
            if *timer <= 0.0 {
                self.player.respawn();
                self.respawn_timer = None;
            }
        }

        self.asteroids.retain(|a| a.alive);
        self.player_lasers.retain(|l| l.alive);
        self.enemy_lasers.retain(|l| l.alive);

        self.particles.update(dt);
        self.shake.update(dt);
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        // Background layer (unshaken)
        self.starfield.draw();

        // Apply screen shake via camera offset
        let (sx, sy) = self.shake.offset();
        set_camera(&Camera2D {
            target: vec2(screen_width() / 2.0 + sx, screen_height() / 2.0 + sy),
            zoom: vec2(2.0 / screen_width(), -2.0 / screen_height()),
            ..Default::default()
        });

        // Game entities (shaken)
        if self.player.alive { self.player.draw(); }
        if self.enemy.alive  { self.enemy.draw(); }
        if self.pickup.alive { self.pickup.draw(); }
        for asteroid in &self.asteroids { asteroid.draw(); }
        for laser in &self.player_lasers { laser.draw(); }
        for laser in &self.enemy_lasers  { laser.draw(); }

        // Particles (shaken, above entities)
        self.particles.draw();

        if self.debug {
            let color = Color::new(0.0, 1.0, 0.0, 0.8);
            if self.player.alive { draw_debug(&self.player, color); }
            if self.enemy.alive  { draw_debug(&self.enemy, color); }
            if self.pickup.alive { draw_debug(&self.pickup, color); }
            for a in &self.asteroids     { draw_debug(a, color); }
            for l in &self.player_lasers { draw_debug(l, color); }
            for l in &self.enemy_lasers  { draw_debug(l, color); }
        }

        // Reset camera for HUD (unshaken)
        set_default_camera();

        draw_text(&format!("Lives: {}", self.player.lives), 10.0, 24.0, 24.0, WHITE);
        draw_text(&format!("Score: {}", self.score), 10.0, 50.0, 24.0, WHITE);

        if !self.player.alive && self.player.lives == 0 {
            let text = "GAME OVER";
            let size = 64.0;
            let dims = measure_text(text, None, size as u16, 1.0);
            draw_text(
                text,
                screen_width()  / 2.0 - dims.width / 2.0,
                screen_height() / 2.0 + dims.height / 2.0,
                size,
                WHITE,
            );
        }
    }
}
