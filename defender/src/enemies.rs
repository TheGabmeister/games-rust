use macroquad::prelude::*;
use crate::constants::*;
use crate::world::{Camera, wrap_direction, wrap_dist_sq};
use crate::astronauts::{Astronaut, AstronautState};
use crate::bullets::Bullet;
use crate::terrain::Terrain;

// ---------------------------------------------------------------------------
// Enemy kind / AI state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
}

#[derive(Clone, Debug)]
pub enum LanderAi {
    Patrolling,
    Descending { target_id: u32 },
    Carrying { astro_id: u32 },
}

#[derive(Clone, Debug)]
pub enum AiState {
    Lander(LanderAi),
    Mutant,
    Bomber { bomb_timer: f32 },
    Pod { phase: f32 },
    Swarmer,
    Baiter { fire_timer: f32 },
}

// ---------------------------------------------------------------------------
// Enemy struct
// ---------------------------------------------------------------------------

pub struct Enemy {
    pub pos: Vec2,
    pub vel: Vec2,
    pub kind: EnemyKind,
    pub ai: AiState,
    pub alive: bool,
    pub id: u32,
    /// Used for rotation / animation state
    pub angle: f32,
}

impl Enemy {
    // --- Constructors ---

    pub fn new_lander(pos: Vec2, id: u32) -> Self {
        Enemy {
            pos,
            vel: Vec2::new(macroquad::rand::gen_range(-LANDER_SPEED, LANDER_SPEED), 0.0),
            kind: EnemyKind::Lander,
            ai: AiState::Lander(LanderAi::Patrolling),
            alive: true,
            id,
            angle: 0.0,
        }
    }

    pub fn new_mutant(pos: Vec2, id: u32) -> Self {
        Enemy {
            pos,
            vel: Vec2::ZERO,
            kind: EnemyKind::Mutant,
            ai: AiState::Mutant,
            alive: true,
            id,
            angle: 0.0,
        }
    }

    pub fn new_bomber(pos: Vec2, id: u32) -> Self {
        let dir = if macroquad::rand::gen_range(0u32, 2) == 0 { 1.0 } else { -1.0 };
        Enemy {
            pos,
            vel: Vec2::new(BOMBER_SPEED * dir, 0.0),
            kind: EnemyKind::Bomber,
            ai: AiState::Bomber { bomb_timer: BOMBER_BOMB_RATE * 0.5 },
            alive: true,
            id,
            angle: 0.0,
        }
    }

    pub fn new_pod(pos: Vec2, id: u32) -> Self {
        Enemy {
            pos,
            vel: Vec2::new(
                macroquad::rand::gen_range(-POD_SPEED, POD_SPEED),
                macroquad::rand::gen_range(-POD_SPEED * 0.5, POD_SPEED * 0.5),
            ),
            kind: EnemyKind::Pod,
            ai: AiState::Pod { phase: macroquad::rand::gen_range(0.0f32, std::f32::consts::TAU) },
            alive: true,
            id,
            angle: 0.0,
        }
    }

    pub fn new_swarmer(pos: Vec2, id: u32) -> Self {
        Enemy {
            pos,
            vel: Vec2::ZERO,
            kind: EnemyKind::Swarmer,
            ai: AiState::Swarmer,
            alive: true,
            id,
            angle: 0.0,
        }
    }

    pub fn new_baiter(pos: Vec2, id: u32) -> Self {
        Enemy {
            pos,
            vel: Vec2::ZERO,
            kind: EnemyKind::Baiter,
            ai: AiState::Baiter { fire_timer: BAITER_FIRE_RATE },
            alive: true,
            id,
            angle: 0.0,
        }
    }

    // --- Update ---

    pub fn update(
        &mut self,
        dt: f32,
        _player_pos: Vec2,
        astronauts: &mut Vec<Astronaut>,
        bullets: &mut Vec<Bullet>,
        terrain: &Terrain,
        time: f32,
    ) {
        match self.kind {
            EnemyKind::Lander => self.update_lander(dt, _player_pos, astronauts, terrain),
            EnemyKind::Mutant => self.update_mutant(dt, _player_pos),
            EnemyKind::Bomber => self.update_bomber(dt, bullets),
            EnemyKind::Pod => self.update_pod(dt, time),
            EnemyKind::Swarmer => self.update_swarmer(dt, _player_pos),
            EnemyKind::Baiter => self.update_baiter(dt, _player_pos, bullets),
        }
        self.pos.x = self.pos.x.rem_euclid(WORLD_WIDTH);
        self.angle += dt * 1.5; // gentle rotation for animated enemies
    }

    fn update_lander(
        &mut self,
        dt: f32,
        _player_pos: Vec2,
        astronauts: &mut Vec<Astronaut>,
        terrain: &Terrain,
    ) {
        let ai = match &self.ai {
            AiState::Lander(s) => s.clone(),
            _ => return,
        };

        match ai {
            LanderAi::Patrolling => {
                // Horizontal wander
                self.pos.x += self.vel.x * dt;

                // Bounce at a slight y drift
                let target_y = WORLD_HEIGHT * 0.35;
                self.pos.y += (target_y - self.pos.y) * dt * 0.3;

                // Search for nearby astronaut
                let mut best: Option<(f32, u32)> = None;
                for a in astronauts.iter() {
                    if !a.alive { continue; }
                    if !matches!(a.state, AstronautState::OnGround) { continue; }
                    let dsq = wrap_dist_sq(self.pos, a.pos);
                    if dsq < LANDER_DETECT_RADIUS * LANDER_DETECT_RADIUS {
                        if best.map_or(true, |(d, _)| dsq < d) {
                            best = Some((dsq, a.id));
                        }
                    }
                }

                if let Some((_, astro_id)) = best {
                    self.ai = AiState::Lander(LanderAi::Descending { target_id: astro_id });
                }
            }

            LanderAi::Descending { target_id } => {
                // Find the target astronaut
                let target = astronauts.iter().find(|a| a.id == target_id && a.alive);
                if let Some(target) = target {
                    if matches!(target.state, AstronautState::OnGround) {
                        let dir = wrap_direction(self.pos, target.pos);
                        self.vel = dir * LANDER_SWOOP_SPEED;
                        self.pos += self.vel * dt;

                        // Reached astronaut?
                        if wrap_dist_sq(self.pos, target.pos) < 20.0 * 20.0 {
                            // Pick up
                            let id = target_id;
                            if let Some(a) = astronauts.iter_mut().find(|a| a.id == id) {
                                a.state = AstronautState::BeingCarried { by_enemy_id: self.id };
                            }
                            self.ai = AiState::Lander(LanderAi::Carrying { astro_id: target_id });
                        }
                    } else {
                        // Astronaut gone, resume patrol
                        self.ai = AiState::Lander(LanderAi::Patrolling);
                        self.vel.x = LANDER_SPEED * if macroquad::rand::gen_range(0u32, 2) == 0 { 1.0 } else { -1.0 };
                    }
                } else {
                    self.ai = AiState::Lander(LanderAi::Patrolling);
                    self.vel.x = LANDER_SPEED;
                }
            }

            LanderAi::Carrying { astro_id } => {
                // Ascend towards top of screen
                self.vel = Vec2::new(self.vel.x * 0.95, -LANDER_ASCEND_SPEED);
                self.pos += self.vel * dt;

                // Keep astronaut glued below
                if let Some(a) = astronauts.iter_mut().find(|a| a.id == astro_id) {
                    a.pos = Vec2::new(self.pos.x, self.pos.y + LANDER_HALF_H + ASTRONAUT_HALF_H + 2.0);
                }

                // Reached top → become mutant (caller handles actual mutation)
                // We signal by leaving pos.y <= ENEMY_TOP_THRESHOLD
            }
        }

        // Clamp vertical within play area for patrolling
        if matches!(&self.ai, AiState::Lander(LanderAi::Patrolling)) {
            let terrain_y = terrain.surface_y(self.pos.x);
            self.pos.y = self.pos.y.clamp(
                SCANNER_HEIGHT + 20.0,
                terrain_y - LANDER_HALF_H - 5.0,
            );
        }
    }

    fn update_mutant(&mut self, dt: f32, player_pos: Vec2) {
        let dir = wrap_direction(self.pos, player_pos);
        self.vel = dir * MUTANT_SPEED;
        self.pos += self.vel * dt;
    }

    fn update_bomber(&mut self, dt: f32, bullets: &mut Vec<Bullet>) {
        self.pos.x += self.vel.x * dt;

        // Slight sine wave vertical drift
        self.pos.y += (self.angle.sin() * 20.0) * dt;
        self.pos.y = self.pos.y.clamp(
            SCANNER_HEIGHT + 20.0,
            WORLD_HEIGHT * 0.6,
        );

        if let AiState::Bomber { ref mut bomb_timer } = self.ai {
            *bomb_timer -= dt;
            if *bomb_timer <= 0.0 {
                *bomb_timer = BOMBER_BOMB_RATE;
                bullets.push(Bullet::new_enemy_bomb(self.pos));
            }
        }
    }

    fn update_pod(&mut self, dt: f32, time: f32) {
        self.pos += self.vel * dt;
        if let AiState::Pod { ref mut phase } = self.ai {
            self.pos.y += (time + *phase).sin() * 15.0 * dt;
            *phase += dt * 0.2;
        }
        self.pos.y = self.pos.y.clamp(SCANNER_HEIGHT + 20.0, WORLD_HEIGHT * 0.5);
    }

    fn update_swarmer(&mut self, dt: f32, player_pos: Vec2) {
        let dir = wrap_direction(self.pos, player_pos);
        self.vel = dir * SWARMER_SPEED;
        self.pos += self.vel * dt;
    }

    fn update_baiter(&mut self, dt: f32, player_pos: Vec2, bullets: &mut Vec<Bullet>) {
        let dir = wrap_direction(self.pos, player_pos);
        self.vel = dir * BAITER_SPEED;
        self.pos += self.vel * dt;

        if let AiState::Baiter { ref mut fire_timer } = self.ai {
            *fire_timer -= dt;
            if *fire_timer <= 0.0 {
                *fire_timer = BAITER_FIRE_RATE;
                let bvel = wrap_direction(self.pos, player_pos) * ENEMY_BULLET_SPEED;
                bullets.push(Bullet::new_enemy_bullet(self.pos, bvel));
            }
        }
    }

    // --- AABB ---

    pub fn aabb(&self) -> Rect {
        let (hw, hh) = match self.kind {
            EnemyKind::Lander => (LANDER_HALF_W, LANDER_HALF_H),
            EnemyKind::Mutant => (12.0, 12.0),
            EnemyKind::Bomber => (12.0, 10.0),
            EnemyKind::Pod => (14.0, 14.0),
            EnemyKind::Swarmer => (7.0, 7.0),
            EnemyKind::Baiter => (10.0, 10.0),
        };
        Rect::new(self.pos.x - hw, self.pos.y - hh, hw * 2.0, hh * 2.0)
    }

    // --- Draw ---

    pub fn draw(&self, camera: &Camera) {
        if !self.alive {
            return;
        }
        let sx = camera.world_to_screen_x(self.pos.x);
        let sy = camera.world_to_screen_y(self.pos.y);

        match self.kind {
            EnemyKind::Lander => draw_lander(sx, sy, &self.ai),
            EnemyKind::Mutant => draw_mutant(sx, sy, self.angle),
            EnemyKind::Bomber => draw_bomber(sx, sy, self.angle),
            EnemyKind::Pod => draw_pod(sx, sy, self.angle),
            EnemyKind::Swarmer => draw_swarmer(sx, sy, &self.vel),
            EnemyKind::Baiter => draw_baiter(sx, sy, self.angle),
        }
    }
}

// ---------------------------------------------------------------------------
// Draw helpers
// ---------------------------------------------------------------------------

fn draw_lander(sx: f32, sy: f32, ai: &AiState) {
    let body_color = RED;
    // Inverted triangle (nose points down)
    draw_triangle(
        Vec2::new(sx, sy + LANDER_HALF_H),          // nose bottom
        Vec2::new(sx - LANDER_HALF_W, sy - LANDER_HALF_H), // top left
        Vec2::new(sx + LANDER_HALF_W, sy - LANDER_HALF_H), // top right
        body_color,
    );
    // Legs
    draw_line(sx - LANDER_HALF_W, sy - LANDER_HALF_H, sx - LANDER_HALF_W - 4.0, sy + 4.0, 2.0, body_color);
    draw_line(sx + LANDER_HALF_W, sy - LANDER_HALF_H, sx + LANDER_HALF_W + 4.0, sy + 4.0, 2.0, body_color);

    // Glow when carrying
    if matches!(ai, AiState::Lander(LanderAi::Carrying { .. })) {
        draw_circle_lines(sx, sy, 18.0, 2.0, Color::new(1.0, 0.8, 0.2, 0.7));
    }
}

fn draw_mutant(sx: f32, sy: f32, angle: f32) {
    // Hexagon, pulsing
    draw_poly(sx, sy, 6, 13.0, angle * 30.0, crate::constants::MAGENTA);
    draw_poly_lines(sx, sy, 6, 13.0, angle * 30.0, 2.0, Color::new(1.0, 0.5, 1.0, 1.0));
    draw_circle(sx, sy, 4.0, Color::new(1.0, 0.3, 1.0, 0.9));
}

fn draw_bomber(sx: f32, sy: f32, angle: f32) {
    // Rotating diamond
    draw_poly(sx, sy, 4, 12.0, angle * 20.0 + 45.0, YELLOW);
    draw_poly_lines(sx, sy, 4, 12.0, angle * 20.0 + 45.0, 2.0, Color::new(1.0, 1.0, 0.3, 1.0));
}

fn draw_pod(sx: f32, sy: f32, angle: f32) {
    draw_circle(sx, sy, 14.0, Color::new(0.5, 0.0, 0.7, 1.0));
    // Spikes
    for i in 0..8 {
        let a = angle * 15.0 + i as f32 * std::f32::consts::TAU / 8.0;
        draw_line(
            sx + a.cos() * 14.0,
            sy + a.sin() * 14.0,
            sx + a.cos() * 20.0,
            sy + a.sin() * 20.0,
            2.0,
            Color::new(0.7, 0.2, 1.0, 1.0),
        );
    }
}

fn draw_swarmer(sx: f32, sy: f32, vel: &Vec2) {
    let angle = vel.y.atan2(vel.x).to_degrees() + 90.0;
    draw_poly(sx, sy, 3, 7.0, angle, RED);
}

fn draw_baiter(sx: f32, sy: f32, angle: f32) {
    draw_poly(sx, sy, 4, 10.0, angle * 90.0, Color::new(1.0, 0.5, 0.0, 1.0));
    draw_poly_lines(sx, sy, 4, 10.0, angle * 90.0, 2.0, Color::new(1.0, 0.8, 0.2, 1.0));
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

pub fn spawn_wave(level: u32, next_id: &mut u32) -> Vec<Enemy> {
    let landers = (6 + level * 2).min(24) as usize;
    let bombers = (2 + level).min(10) as usize;
    let pods = (1 + level / 2).min(6) as usize;

    let mut enemies = Vec::new();

    let spacing = WORLD_WIDTH / (landers + bombers + pods) as f32;
    let mut x = spacing * 0.5;

    for _ in 0..landers {
        let y = macroquad::rand::gen_range(SCANNER_HEIGHT + 40.0, WORLD_HEIGHT * 0.5);
        enemies.push(Enemy::new_lander(Vec2::new(x, y), *next_id));
        *next_id += 1;
        x += spacing;
    }
    for _ in 0..bombers {
        let y = macroquad::rand::gen_range(SCANNER_HEIGHT + 40.0, WORLD_HEIGHT * 0.4);
        enemies.push(Enemy::new_bomber(Vec2::new(x, y), *next_id));
        *next_id += 1;
        x += spacing;
    }
    for _ in 0..pods {
        let y = macroquad::rand::gen_range(SCANNER_HEIGHT + 30.0, WORLD_HEIGHT * 0.35);
        enemies.push(Enemy::new_pod(Vec2::new(x, y), *next_id));
        *next_id += 1;
        x += spacing;
    }

    enemies
}

pub fn spawn_mutant_wave(count: usize, next_id: &mut u32) -> Vec<Enemy> {
    (0..count)
        .map(|_| {
            let x = macroquad::rand::gen_range(0.0f32, WORLD_WIDTH);
            let y = macroquad::rand::gen_range(SCANNER_HEIGHT + 40.0, WORLD_HEIGHT * 0.6);
            let e = Enemy::new_mutant(Vec2::new(x, y), *next_id);
            *next_id += 1;
            e
        })
        .collect()
}

/// Color used for explosion particles.
pub fn enemy_explosion_color(kind: EnemyKind) -> Color {
    match kind {
        EnemyKind::Lander => RED,
        EnemyKind::Mutant => crate::constants::MAGENTA,
        EnemyKind::Bomber => YELLOW,
        EnemyKind::Pod => Color::new(0.6, 0.0, 1.0, 1.0),
        EnemyKind::Swarmer => Color::new(1.0, 0.3, 0.3, 1.0),
        EnemyKind::Baiter => ORANGE,
    }
}
