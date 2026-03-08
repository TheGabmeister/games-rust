use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

use crate::assets::Assets;
use crate::asteroid::Asteroid;
use crate::input::InputState;
use crate::collidable::overlaps;
use crate::enemy::Enemy;
use crate::laser::Laser;
use crate::pickup::Pickup;
use crate::player::Player;

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
    pub should_quit: bool,
    score:         u32,
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
            should_quit:   false,
            score:         0,
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        if input.quit {
            self.should_quit = true;
        }

        if self.player.alive {
            self.player.update(dt, input);

            if input.shoot {
                self.player_lasers.push(Laser::new(
                    self.player.x, self.player.y, -500.0, self.laser_texture.clone(),
                ));
                play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }
        }

        if self.enemy.alive {
            if self.enemy.update(dt) {
                self.enemy_lasers.push(Laser::new(
                    self.enemy.x, self.enemy.y, 500.0, self.enemy_laser_texture.clone(),
                ));
                play_sound(&self.sfx_laser, PlaySoundParams { looped: false, volume: 1.0 });
            }

            for laser in &mut self.player_lasers {
                if laser.alive && overlaps(laser, &self.enemy) {
                    laser.alive = false;
                    self.enemy.alive = false;
                }
            }
        }

        if self.player.alive {
            for laser in &mut self.enemy_lasers {
                if laser.alive && overlaps(laser, &self.player) {
                    laser.alive = false;
                    self.player.alive = false;
                }
            }

            if self.pickup.alive && overlaps(&self.player, &self.pickup) {
                self.pickup.alive = false;
                self.player.lives += 1;
            }
        }

        for asteroid in &mut self.asteroids {
            for laser in &mut self.player_lasers {
                if laser.alive && asteroid.alive && overlaps(laser, asteroid) {
                    laser.alive = false;
                    asteroid.alive = false;
                    self.score += 100;
                    play_sound(&self.sfx_bump, PlaySoundParams { looped: false, volume: 1.0 });
                }
            }
        }
        self.asteroids.retain(|a| a.alive);
        for asteroid in &mut self.asteroids { asteroid.update(dt); }
        for laser in &mut self.player_lasers { laser.update(dt); }
        for laser in &mut self.enemy_lasers  { laser.update(dt); }
        self.player_lasers.retain(|l| l.alive);
        self.enemy_lasers.retain(|l| l.alive);
    }

    pub fn draw(&self) {
        clear_background(BLACK);
        if self.player.alive { self.player.draw(); }
        if self.enemy.alive  { self.enemy.draw(); }
        if self.pickup.alive { self.pickup.draw(); }
        for asteroid in &self.asteroids { asteroid.draw(); }
        for laser in &self.player_lasers { laser.draw(); }
        for laser in &self.enemy_lasers  { laser.draw(); }

        draw_text(&format!("Lives: {}", self.player.lives), 10.0, 24.0, 24.0, WHITE);
        draw_text(&format!("Score: {}", self.score), 10.0, 50.0, 24.0, WHITE);
    }
}
