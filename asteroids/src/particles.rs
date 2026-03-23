use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: Color,
    lifetime: f32,
    max_lifetime: f32,
    size: f32,
}

struct Flash {
    x: f32,
    y: f32,
    max_radius: f32,
    lifetime: f32,
    max_lifetime: f32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    flashes: Vec<Flash>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self { particles: Vec::new(), flashes: Vec::new() }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        for f in &mut self.flashes {
            f.lifetime -= dt;
        }
        self.flashes.retain(|f| f.lifetime > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let t = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let color = Color::new(p.color.r, p.color.g, p.color.b, t);
            draw_circle(p.x, p.y, p.size * t, color);
        }

        for f in &self.flashes {
            let t = (f.lifetime / f.max_lifetime).clamp(0.0, 1.0);
            let radius = f.max_radius * (1.0 - t); // expands outward
            let alpha = t * 0.8; // fades as it expands
            draw_circle(f.x, f.y, radius, Color::new(1.0, 1.0, 1.0, alpha));
        }
    }

    pub fn spawn_burst(
        &mut self,
        x: f32,
        y: f32,
        count: usize,
        color: Color,
        speed: f32,
        lifetime: f32,
        size: f32,
    ) {
        for _ in 0..count {
            let angle = gen_range(0.0, TAU);
            let spd = gen_range(speed * 0.3, speed);
            let lt = gen_range(lifetime * 0.5, lifetime);
            let sz = gen_range(size * 0.5, size);
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * spd,
                vy: angle.sin() * spd,
                color,
                lifetime: lt,
                max_lifetime: lt,
                size: sz,
            });
        }
    }

    pub fn spawn_flash(&mut self, x: f32, y: f32, max_radius: f32, lifetime: f32) {
        self.flashes.push(Flash {
            x,
            y,
            max_radius,
            lifetime,
            max_lifetime: lifetime,
        });
    }

    pub fn spawn_trail(&mut self, x: f32, y: f32, color: Color) {
        self.particles.push(Particle {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            color,
            lifetime: 0.08,
            max_lifetime: 0.08,
            size: 2.0,
        });
    }

    pub fn spawn_thrust(&mut self, x: f32, y: f32, angle: f32, count: usize) {
        let base_angle = angle + PI;
        let spawn_x = x - angle.sin() * 18.0;
        let spawn_y = y + angle.cos() * 18.0;
        for _ in 0..count {
            let a = base_angle + gen_range(-0.3, 0.3);
            let spd = gen_range(80.0, 180.0);
            let lt = gen_range(0.1, 0.3);
            self.particles.push(Particle {
                x: spawn_x,
                y: spawn_y,
                vx: a.sin() * spd,
                vy: -a.cos() * spd,
                color: Color::new(1.0, gen_range(0.4, 0.8), 0.0, 1.0),
                lifetime: lt,
                max_lifetime: lt,
                size: gen_range(1.5, 3.0),
            });
        }
    }
}
