use macroquad::audio::play_sound_once;
use macroquad::prelude::*;
use crate::assets::AssetManager;
use crate::debug::draw_debug_ui;
use crate::entities::{Entity, EntityKind};
use crate::render::{draw_entity, draw_overlay};
use crate::state::GameState;
use crate::systems::{apply_player_input, check_collisions, clamp_to_screen};

const PLAYER_SIZE: Vec2 = vec2(48.0, 48.0);
const ENEMY_SIZE: Vec2 = vec2(48.0, 48.0);

pub struct Game {
    pub entities: Vec<Entity>,
    pub state: GameState,
    pub assets: AssetManager,
    pub quit: bool,
    pub debug: bool,
    next_id: u32,
}

impl Game {
    pub async fn new() -> Self {
        let assets = AssetManager::load("assets/assets.json").await;

        let mut game = Self {
            entities: Vec::new(),
            state: GameState::Playing,
            assets,
            quit: false,
            debug: false,
            next_id: 0,
        };

        let id = game.alloc_id();
        let pos = vec2(
            screen_width() / 2.0 - PLAYER_SIZE.x / 2.0,
            screen_height() / 2.0 - PLAYER_SIZE.y / 2.0,
        );
        let player = Entity::new(id, EntityKind::Player, pos, PLAYER_SIZE).with_texture("player");
        game.entities.push(player);

        let id = game.alloc_id();
        let enemy_pos = vec2(screen_width() * 0.7, screen_height() * 0.2);
        let enemy = Entity::new(id, EntityKind::Enemy, enemy_pos, ENEMY_SIZE).with_texture("enemy");
        game.entities.push(enemy);

        game
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn update(&mut self) {
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Escape) {
            self.quit = true;
        }
        if is_key_pressed(KeyCode::F1) {
            self.debug = !self.debug;
        }
        if is_key_pressed(KeyCode::P) {
            match self.state {
                GameState::Playing => self.state = GameState::Paused,
                GameState::Paused  => self.state = GameState::Playing,
                _ => {}
            }
        }

        if self.state == GameState::Playing {
            apply_player_input(&mut self.entities);
            for entity in &mut self.entities {
                entity.position += entity.velocity * dt;
                clamp_to_screen(entity);
            }
            self.check_player_collision();
        }
    }

    fn check_player_collision(&mut self) {
        let collisions = check_collisions(&self.entities);

        let player_id = self.entities.iter()
            .find(|e| e.kind == EntityKind::Player && e.active)
            .map(|e| e.id);

        let Some(player_id) = player_id else { return };

        let hit = collisions.iter().any(|(a, b)| *a == player_id || *b == player_id);
        if !hit {
            return;
        }

        if let Some(player) = self.entities.iter_mut().find(|e| e.id == player_id) {
            player.active = false;
        }
        if let Some(sound) = self.assets.sound("lose") {
            play_sound_once(sound);
        }
        self.state = GameState::GameOver;
    }

    pub fn draw(&self) {
        clear_background(Color::from_hex(0x0d0d1a));

        for entity in &self.entities {
            draw_entity(entity, &self.assets, self.debug);
        }

        draw_overlay(&self.state);
        draw_debug_ui(&self.entities, self.debug);

        draw_text("F1: toggle debug", 6.0, screen_height() - 8.0, 14.0, DARKGRAY);
    }
}
