use macroquad::prelude::*;
use crate::constants::*;
use crate::world::Camera;
use crate::terrain::Terrain;
use crate::player::{Player, PlayerAction};
use crate::enemies::{
    Enemy, EnemyKind, AiState, LanderAi,
    spawn_wave, spawn_mutant_wave, enemy_explosion_color,
};
use crate::astronauts::{Astronaut, AstronautState, spawn_astronauts};
use crate::bullets::{Bullet, BulletOwner};
use crate::particles::ParticleSystem;
use crate::scanner::Scanner;
use crate::scoring::Scoring;
use crate::audio::{AudioSystem, SoundEffect};
use crate::collision::aabb_overlap_wrapped;

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

pub enum GamePhase {
    Title,
    Playing,
    PlayerDead { timer: f32 },
    LevelComplete { timer: f32 },
    GameOver,
}

// ---------------------------------------------------------------------------
// Game
// ---------------------------------------------------------------------------

pub struct Game {
    pub phase: GamePhase,
    pub camera: Camera,
    pub terrain: Terrain,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub astronauts: Vec<Astronaut>,
    pub particles: ParticleSystem,
    pub scoring: Scoring,
    pub audio: AudioSystem,

    pub baiter_timer: f32,
    pub planet_destroyed: bool,
    pub next_id: u32,
    pub time: f32,
    pub game_over_delay: f32,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Game {
            phase: GamePhase::Title,
            camera: Camera::new(),
            terrain: Terrain::generate(1),
            player: Player::new(WORLD_WIDTH / 2.0, WORLD_HEIGHT / 2.0),
            enemies: Vec::new(),
            bullets: Vec::new(),
            astronauts: Vec::new(),
            particles: ParticleSystem::new(),
            scoring: Scoring::new(),
            audio: AudioSystem::new(),
            baiter_timer: BAITER_SPAWN_TIME,
            planet_destroyed: false,
            next_id: 1,
            time: 0.0,
            game_over_delay: 0.0,
        };
        g.reset_level();
        g
    }

    pub fn full_reset(&mut self) {
        self.scoring = Scoring::new();
        self.scoring.level = 1;
        self.next_id = 1;
        self.reset_level();
    }

    pub fn reset_level(&mut self) {
        let level = self.scoring.level;
        self.terrain = Terrain::generate(level * 31337 + 1);
        self.enemies = spawn_wave(level, &mut self.next_id);
        self.astronauts = spawn_astronauts(&self.terrain, ASTRONAUT_COUNT, &mut self.next_id);
        self.bullets.clear();
        self.particles = ParticleSystem::new();
        self.baiter_timer = BAITER_SPAWN_TIME;
        self.planet_destroyed = false;

        let start_x = WORLD_WIDTH / 2.0;
        let start_y = self.terrain.surface_y(start_x) - 80.0;
        self.player = Player::new(start_x, start_y);
        self.camera.follow(self.player.pos.x);
    }

    // -----------------------------------------------------------------------
    // Update
    // -----------------------------------------------------------------------

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        match self.phase {
            GamePhase::Title => self.update_title(),
            GamePhase::Playing => self.update_playing(dt),
            GamePhase::PlayerDead { ref mut timer } => {
                *timer -= dt;
                if *timer <= 0.0 {
                    if self.scoring.lives == 0 {
                        self.phase = GamePhase::GameOver;
                        self.game_over_delay = 1.5;
                    } else {
                        let start_x = WORLD_WIDTH / 2.0;
                        let start_y = self.terrain.surface_y(start_x) - 80.0;
                        self.player = Player::new(start_x, start_y);
                        self.camera.follow(self.player.pos.x);
                        self.bullets.retain(|b| matches!(b.owner, BulletOwner::Player));
                        self.phase = GamePhase::Playing;
                    }
                }
            }
            GamePhase::LevelComplete { ref mut timer } => {
                *timer -= dt;
                self.particles.update(dt);
                if *timer <= 0.0 {
                    self.scoring.level += 1;
                    self.reset_level();
                    self.phase = GamePhase::Playing;
                }
            }
            GamePhase::GameOver => {
                if self.game_over_delay > 0.0 {
                    self.game_over_delay -= dt;
                } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.phase = GamePhase::Title;
                }
            }
        }
    }

    fn update_title(&mut self) {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.full_reset();
            self.phase = GamePhase::Playing;
        }
    }

    fn update_playing(&mut self, dt: f32) {
        // --- Player ---
        let actions = self.player.update(dt, &self.terrain);
        self.process_player_actions(actions);
        self.camera.follow(self.player.pos.x);

        // --- Bullets ---
        for b in self.bullets.iter_mut() {
            b.update(dt);
        }
        self.bullets.retain(|b| b.alive);

        // --- Enemies (two-pass: update then post-process) ---
        let player_pos = self.player.pos;
        let mut new_bullets: Vec<Bullet> = Vec::new();

        for e in self.enemies.iter_mut() {
            if !e.alive { continue; }
            e.update(dt, player_pos, &mut self.astronauts, &mut new_bullets, &self.terrain, self.time);
        }
        self.bullets.extend(new_bullets);

        // Landers carrying astronaut that reached the top → become mutant
        let mut to_mutate: Vec<u32> = Vec::new();
        for e in self.enemies.iter() {
            if !e.alive { continue; }
            if e.kind == EnemyKind::Lander {
                if let AiState::Lander(LanderAi::Carrying { .. }) = &e.ai {
                    if e.pos.y <= ENEMY_TOP_THRESHOLD {
                        to_mutate.push(e.id);
                    }
                }
            }
        }
        for eid in to_mutate {
            if let Some(e) = self.enemies.iter_mut().find(|e| e.id == eid) {
                e.alive = false;
                // Free / despawn carried astronaut
                if let AiState::Lander(LanderAi::Carrying { astro_id }) = e.ai.clone() {
                    if let Some(a) = self.astronauts.iter_mut().find(|a| a.id == astro_id) {
                        a.alive = false; // astronaut consumed by mutation
                    }
                }
                let new_mutant = Enemy::new_mutant(e.pos, self.next_id);
                self.next_id += 1;
                self.enemies.push(new_mutant);
            }
        }
        self.enemies.retain(|e| e.alive);

        // --- Astronauts ---
        for a in self.astronauts.iter_mut() {
            if a.alive {
                a.update(dt, &self.terrain);
            }
        }

        // --- Baiter timer ---
        self.baiter_timer -= dt;
        if self.baiter_timer <= 0.0 {
            self.baiter_timer = BAITER_SPAWN_TIME;
            let baiter = Enemy::new_baiter(self.player.pos, self.next_id);
            self.next_id += 1;
            self.enemies.push(baiter);
        }

        // --- Particles ---
        self.particles.update(dt);

        // --- Collisions ---
        if self.player.alive {
            self.resolve_collisions();
        }

        // --- Planet destruction check ---
        let live_astros = self.astronauts.iter()
            .filter(|a| a.alive && !matches!(a.state, AstronautState::Safe))
            .count();
        if live_astros == 0 && !self.planet_destroyed && !self.astronauts.is_empty() {
            self.planet_destroyed = true;
            self.enemies.clear();
            let count = (8 + self.scoring.level as usize * 2).min(40);
            let mut mutants = spawn_mutant_wave(count, &mut self.next_id);
            self.enemies.append(&mut mutants);
        }

        // --- Level complete ---
        if self.enemies.is_empty() {
            if matches!(self.phase, GamePhase::Playing) {
                self.phase = GamePhase::LevelComplete { timer: LEVEL_COMPLETE_DURATION };
            }
        }
    }

    fn process_player_actions(&mut self, actions: Vec<PlayerAction>) {
        for action in actions {
            match action {
                PlayerAction::FireBullet { pos, vel } => {
                    let player_bullets = self.bullets.iter()
                        .filter(|b| matches!(b.owner, BulletOwner::Player))
                        .count();
                    if player_bullets < MAX_PLAYER_BULLETS {
                        self.bullets.push(Bullet::new_player(pos, vel));
                        self.audio.play(SoundEffect::PlayerShoot);
                    }
                }
                PlayerAction::SmartBomb => {
                    self.particles.spawn_smart_bomb_flash();
                    self.audio.play(SoundEffect::SmartBomb);

                    let cam = &self.camera;
                    let mut pending_spawns: Vec<Enemy> = Vec::new();
                    for e in self.enemies.iter_mut() {
                        if !e.alive { continue; }
                        if cam.is_visible(e.pos.x, 30.0) {
                            e.alive = false;
                            let color = enemy_explosion_color(e.kind);
                            self.particles.spawn_explosion(e.pos, color, 12);
                            let pts = score_for_kind(e.kind);
                            self.scoring.add(pts);
                            self.particles.spawn_score_text(e.pos, pts);

                            // Pod splits
                            if e.kind == EnemyKind::Pod {
                                let n = macroquad::rand::gen_range(
                                    POD_SWARMER_COUNT_MIN as u32,
                                    POD_SWARMER_COUNT_MAX as u32 + 1,
                                ) as usize;
                                for _ in 0..n {
                                    pending_spawns.push(Enemy::new_swarmer(e.pos, self.next_id));
                                    self.next_id += 1;
                                }
                            }
                            // Free carried astronaut
                            if e.kind == EnemyKind::Lander {
                                if let AiState::Lander(LanderAi::Carrying { astro_id }) = &e.ai {
                                    let aid = *astro_id;
                                    if let Some(a) = self.astronauts.iter_mut().find(|a| a.id == aid) {
                                        a.state = AstronautState::Falling { vel_y: 0.0 };
                                    }
                                }
                            }
                        }
                    }
                    self.enemies.retain(|e| e.alive);
                    self.enemies.extend(pending_spawns);
                }
                PlayerAction::Hyperspace => {
                    self.audio.play(SoundEffect::Hyperspace);
                    if macroquad::rand::gen_range(0.0f32, 1.0) < HYPERSPACE_DEATH_CHANCE {
                        self.trigger_player_death();
                    } else {
                        self.player.pos.x = macroquad::rand::gen_range(0.0f32, WORLD_WIDTH);
                        self.player.pos.y = macroquad::rand::gen_range(
                            SCANNER_HEIGHT + 50.0,
                            self.terrain.surface_y(self.player.pos.x) - 50.0,
                        );
                        self.player.invincible_timer = PLAYER_INVINCIBLE_TIME;
                        self.camera.follow(self.player.pos.x);
                    }
                }
            }
        }
    }

    fn resolve_collisions(&mut self) {
        // --- Player bullets vs enemies ---
        let mut kill_enemy_ids: Vec<u32> = Vec::new();

        'bullets: for b in self.bullets.iter_mut() {
            if !b.alive || !matches!(b.owner, BulletOwner::Player) {
                continue;
            }
            for e in self.enemies.iter() {
                if !e.alive { continue; }
                if aabb_overlap_wrapped(b.aabb(), e.aabb()) {
                    b.alive = false;
                    kill_enemy_ids.push(e.id);
                    continue 'bullets;
                }
            }
        }

        let mut pending_spawns: Vec<Enemy> = Vec::new();
        for eid in kill_enemy_ids {
            if let Some(e) = self.enemies.iter_mut().find(|e| e.id == eid && e.alive) {
                e.alive = false;
                let color = enemy_explosion_color(e.kind);
                self.particles.spawn_explosion(e.pos, color, 12);
                let pts = score_for_kind(e.kind);
                self.scoring.add(pts);
                self.particles.spawn_score_text(e.pos, pts);
                self.audio.play(SoundEffect::EnemyExplode);

                if e.kind == EnemyKind::Pod {
                    let n = macroquad::rand::gen_range(
                        POD_SWARMER_COUNT_MIN as u32,
                        POD_SWARMER_COUNT_MAX as u32 + 1,
                    ) as usize;
                    for _ in 0..n {
                        pending_spawns.push(Enemy::new_swarmer(e.pos, self.next_id));
                        self.next_id += 1;
                    }
                }

                // Free carried astronaut
                if e.kind == EnemyKind::Lander {
                    if let AiState::Lander(LanderAi::Carrying { astro_id }) = &e.ai.clone() {
                        let aid = *astro_id;
                        if let Some(a) = self.astronauts.iter_mut().find(|a| a.id == aid) {
                            a.state = AstronautState::Falling { vel_y: 0.0 };
                        }
                    }
                }
            }
        }
        self.enemies.retain(|e| e.alive);
        self.enemies.extend(pending_spawns);

        // Early exit if player died in this pass
        if !self.player.alive || self.player.invincible_timer > 0.0 {
            return;
        }

        // --- Enemy bombs/bullets vs player ---
        for b in self.bullets.iter_mut() {
            if !b.alive || !matches!(b.owner, BulletOwner::Enemy) { continue; }
            if aabb_overlap_wrapped(b.aabb(), self.player.aabb()) {
                b.alive = false;
                self.trigger_player_death();
                return;
            }
        }

        // --- Enemy contact vs player ---
        for e in self.enemies.iter() {
            if !e.alive { continue; }
            if aabb_overlap_wrapped(e.aabb(), self.player.aabb()) {
                self.trigger_player_death();
                return;
            }
        }

        // --- Player catches falling astronaut ---
        for a in self.astronauts.iter_mut() {
            if !a.alive || !a.is_catchable() { continue; }
            if aabb_overlap_wrapped(a.aabb(), self.player.aabb()) {
                a.state = AstronautState::Safe;
                self.player.carried_astronaut = Some(a.id);
                let pts = ASTRONAUT_CATCH_SCORE_BASE * (self.scoring.level + 1);
                self.scoring.add(pts);
                self.particles.spawn_score_text(a.pos, pts);
                self.audio.play(SoundEffect::AstronautCatch);
                break;
            }
        }

        // --- Deposit astronaut when player is near ground ---
        if let Some(aid) = self.player.carried_astronaut {
            let terrain_y = self.terrain.surface_y(self.player.pos.x);
            if self.player.pos.y >= terrain_y - PLAYER_HALF_H - 20.0 {
                if let Some(a) = self.astronauts.iter_mut().find(|a| a.id == aid) {
                    a.pos = Vec2::new(self.player.pos.x, terrain_y - ASTRONAUT_HALF_H);
                    a.state = AstronautState::OnGround;
                }
                self.player.carried_astronaut = None;
            }
        }
    }

    fn trigger_player_death(&mut self) {
        if !self.player.alive { return; }
        self.player.alive = false;
        self.particles.spawn_explosion(self.player.pos, crate::constants::CYAN, 20);
        self.audio.play(SoundEffect::PlayerExplode);
        self.scoring.lose_life(); // decrement lives
        self.phase = GamePhase::PlayerDead { timer: PLAYER_DEAD_DURATION };
    }

    // -----------------------------------------------------------------------
    // Draw
    // -----------------------------------------------------------------------

    pub fn draw(&self) {
        clear_background(BLACK);

        match &self.phase {
            GamePhase::Title => self.draw_title(),
            GamePhase::GameOver => self.draw_game_over(),
            GamePhase::LevelComplete { timer } => {
                self.draw_playing();
                self.draw_level_complete(*timer);
            }
            _ => self.draw_playing(),
        }
    }

    fn draw_playing(&self) {
        self.terrain.draw(&self.camera);
        for a in &self.astronauts {
            if a.alive { a.draw(&self.camera); }
        }
        for b in &self.bullets {
            b.draw(&self.camera);
        }
        for e in &self.enemies {
            e.draw(&self.camera);
        }
        self.player.draw(&self.camera);
        self.particles.draw(&self.camera);
        Scanner::draw(&self.camera, &self.player, &self.enemies, &self.astronauts, &self.terrain);
        self.scoring.draw_hud(self.player.smart_bombs);

        // Planet destroyed warning
        if self.planet_destroyed {
            let sw = screen_width();
            let warn = "PLANET DESTROYED - MUTANT WAVE!";
            let tw = measure_text(warn, None, 22, 1.0).width;
            draw_text(warn, sw / 2.0 - tw / 2.0, SCANNER_HEIGHT + 30.0, 22.0,
                      Color::new(1.0, 0.3, 0.3, 1.0));
        }

        // Dead player flash
        if !self.player.alive {
            if let GamePhase::PlayerDead { timer } = &self.phase {
                let flash = (*timer * 5.0) as u32 % 2 == 0;
                if flash {
                    let sw = screen_width();
                    let msg = "SHIP DESTROYED";
                    let tw = measure_text(msg, None, 30, 1.0).width;
                    draw_text(msg, sw / 2.0 - tw / 2.0, screen_height() / 2.0, 30.0, RED);
                }
            }
        }
    }

    fn draw_title(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = sw / 2.0;

        // Title
        let title = "DEFENDER";
        let tw = measure_text(title, None, 80, 1.0).width;
        draw_text(title, cx - tw / 2.0, sh * 0.30, 80.0, Color::new(0.3, 0.8, 1.0, 1.0));

        // Subtitle
        let sub = "Classic Arcade  1981";
        let sw2 = measure_text(sub, None, 22, 1.0).width;
        draw_text(sub, cx - sw2 / 2.0, sh * 0.42, 22.0, Color::new(0.5, 0.5, 0.5, 1.0));

        // Controls
        let controls = [
            "Arrow Keys  —  Move",
            "Z / LCtrl   —  Fire",
            "X           —  Smart Bomb",
            "C / LShift  —  Hyperspace",
        ];
        for (i, line) in controls.iter().enumerate() {
            let lw = measure_text(line, None, 20, 1.0).width;
            draw_text(line, cx - lw / 2.0, sh * 0.55 + i as f32 * 28.0, 20.0, LIGHTGRAY);
        }

        // Objective
        let obj = "Protect the astronauts! Catch them when they fall!";
        let ow = measure_text(obj, None, 18, 1.0).width;
        draw_text(obj, cx - ow / 2.0, sh * 0.80, 18.0, Color::new(0.2, 1.0, 0.3, 1.0));

        // Prompt
        let blink = (self.time * 2.0) as u32 % 2 == 0;
        if blink {
            let prompt = "PRESS ENTER OR SPACE TO START";
            let pw = measure_text(prompt, None, 24, 1.0).width;
            draw_text(prompt, cx - pw / 2.0, sh * 0.90, 24.0, WHITE);
        }
    }

    fn draw_game_over(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = sw / 2.0;

        let go = "GAME OVER";
        let gw = measure_text(go, None, 70, 1.0).width;
        draw_text(go, cx - gw / 2.0, sh * 0.40, 70.0, RED);

        let score_str = format!("Score: {}", self.scoring.score);
        let ssw = measure_text(&score_str, None, 30, 1.0).width;
        draw_text(&score_str, cx - ssw / 2.0, sh * 0.55, 30.0, WHITE);

        let hi_str = format!("High Score: {}", self.scoring.high_score);
        let hsw = measure_text(&hi_str, None, 24, 1.0).width;
        draw_text(&hi_str, cx - hsw / 2.0, sh * 0.64, 24.0, YELLOW);

        if self.game_over_delay <= 0.0 {
            let blink = (self.time * 2.0) as u32 % 2 == 0;
            if blink {
                let prompt = "PRESS ENTER TO RETURN TO TITLE";
                let pw = measure_text(prompt, None, 22, 1.0).width;
                draw_text(prompt, cx - pw / 2.0, sh * 0.80, 22.0, LIGHTGRAY);
            }
        }
    }

    fn draw_level_complete(&self, _timer: f32) {
        let sw = screen_width();
        let sh = screen_height();
        let cx = sw / 2.0;

        let msg = format!("LEVEL {} COMPLETE!", self.scoring.level);
        let mw = measure_text(&msg, None, 48, 1.0).width;
        draw_text(&msg, cx - mw / 2.0, sh / 2.0 - 30.0, 48.0, YELLOW);

        let next = format!("Prepare for Level {}...", self.scoring.level + 1);
        let nw = measure_text(&next, None, 26, 1.0).width;
        draw_text(&next, cx - nw / 2.0, sh / 2.0 + 20.0, 26.0, LIGHTGRAY);
    }
}

fn score_for_kind(kind: EnemyKind) -> u32 {
    match kind {
        EnemyKind::Lander => SCORE_LANDER,
        EnemyKind::Mutant => SCORE_MUTANT,
        EnemyKind::Baiter => SCORE_BAITER,
        EnemyKind::Bomber => SCORE_BOMBER,
        EnemyKind::Pod => SCORE_POD,
        EnemyKind::Swarmer => SCORE_SWARMER,
    }
}
