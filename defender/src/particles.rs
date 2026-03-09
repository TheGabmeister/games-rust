use macroquad::prelude::*;
use crate::constants::SCANNER_HEIGHT;
use crate::world::Camera;

enum ParticleKind {
    Spark { color: Color },
    SmartBombFlash,
    ScoreText { text: String },
}

struct Particle {
    pub pos: Vec2,     // world-space for sparks, screen-space for flash
    pub vel: Vec2,
    kind: ParticleKind,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem { particles: Vec::new() }
    }

    pub fn spawn_explosion(&mut self, world_pos: Vec2, color: Color, count: usize) {
        use std::f32::consts::TAU;
        for i in 0..count {
            let angle = (i as f32 / count as f32) * TAU
                + macroquad::rand::gen_range(-0.3f32, 0.3);
            let speed = macroquad::rand::gen_range(60.0f32, 200.0);
            let life = macroquad::rand::gen_range(0.3f32, 0.9);
            self.particles.push(Particle {
                pos: world_pos,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                kind: ParticleKind::Spark { color },
                lifetime: life,
                max_lifetime: life,
            });
        }
    }

    pub fn spawn_smart_bomb_flash(&mut self) {
        self.particles.push(Particle {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            kind: ParticleKind::SmartBombFlash,
            lifetime: 0.45,
            max_lifetime: 0.45,
        });
    }

    pub fn spawn_score_text(&mut self, world_pos: Vec2, score: u32) {
        self.particles.push(Particle {
            pos: world_pos,
            vel: Vec2::new(0.0, -35.0),
            kind: ParticleKind::ScoreText { text: format!("+{}", score) },
            lifetime: 1.0,
            max_lifetime: 1.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            match &p.kind {
                ParticleKind::SmartBombFlash => {}
                _ => {
                    p.pos.x = (p.pos.x + p.vel.x * dt).rem_euclid(crate::constants::WORLD_WIDTH);
                    p.pos.y += p.vel.y * dt;
                }
            }
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);
    }

    pub fn draw(&self, camera: &Camera) {
        for p in &self.particles {
            match &p.kind {
                ParticleKind::Spark { color } => {
                    let alpha = (p.lifetime / p.max_lifetime).max(0.0);
                    let c = Color::new(color.r, color.g, color.b, alpha);
                    let sx = camera.world_to_screen_x(p.pos.x);
                    let sy = camera.world_to_screen_y(p.pos.y);
                    draw_circle(sx, sy, 2.5, c);
                }
                ParticleKind::SmartBombFlash => {
                    let alpha = (p.lifetime / p.max_lifetime * 0.85).max(0.0);
                    draw_rectangle(
                        0.0,
                        SCANNER_HEIGHT,
                        screen_width(),
                        screen_height() - SCANNER_HEIGHT,
                        Color::new(1.0, 1.0, 1.0, alpha),
                    );
                }
                ParticleKind::ScoreText { text } => {
                    let alpha = (p.lifetime / p.max_lifetime).max(0.0);
                    let sx = camera.world_to_screen_x(p.pos.x);
                    let sy = camera.world_to_screen_y(p.pos.y);
                    draw_text(text, sx - 12.0, sy, 18.0, Color::new(1.0, 1.0, 0.2, alpha));
                }
            }
        }
    }
}
