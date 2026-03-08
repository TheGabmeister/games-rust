use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

use crate::assets::Assets;
use crate::collidable::overlaps;
use crate::enemy::Enemy;
use crate::laser::Laser;
use crate::pickup::Pickup;
use crate::player::Player;

pub struct Game {
    player:        Player,
    enemy:         Enemy,
    pickup:        Pickup,
    player_lasers: Vec<Laser>,
    enemy_lasers:  Vec<Laser>,
    laser_texture:       Texture2D,
    enemy_laser_texture: Texture2D,
    sfx_laser:     Sound,
    pub should_quit: bool,
}

impl Game {
    pub fn new(assets: &Assets) -> Self {
        Self {
            player:        Player::new(assets.player_ship.clone()),
            enemy:         Enemy::new(assets.enemy_ufo_green.clone()),
            pickup:        Pickup::new(600.0, 450.0, assets.pill_blue.clone()),
            player_lasers: Vec::new(),
            enemy_lasers:  Vec::new(),
            laser_texture:       assets.player_laser.clone(),
            enemy_laser_texture: assets.enemy_laser.clone(),
            sfx_laser:     assets.sfx_laser.clone(),
            should_quit:   false,
        }
    }

    pub fn update(&mut self, dt: f32) {
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
        for laser in &self.player_lasers { laser.draw(); }
        for laser in &self.enemy_lasers  { laser.draw(); }

        draw_text(&format!("Lives: {}", self.player.lives), 10.0, 24.0, 24.0, WHITE);
    }
}
