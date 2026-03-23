use macroquad::prelude::*;
use macroquad::color::hsl_to_rgb;
use macroquad::rand::gen_range;
use macroquad::window::miniquad::*;

use crate::palette;
use crate::scene::Scene;
use crate::shaders;

const MAX_PARTICLES: usize = 2000;
const MAX_EMBERS: usize = 200;
const SPAWN_RATE: usize = 25;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
}

impl Particle {
    fn spawn(cx: f32, cy: f32, is_ember: bool) -> Self {
        if is_ember {
            Self {
                pos: vec2(cx + gen_range(-100.0, 100.0), cy),
                vel: vec2(gen_range(-20.0, 20.0), gen_range(-200.0, -100.0)),
                life: 1.0,
                max_life: gen_range(2.0, 3.0),
                size: gen_range(1.5, 3.5),
            }
        } else {
            Self {
                pos: vec2(cx + gen_range(-100.0, 100.0), cy),
                vel: vec2(gen_range(-30.0, 30.0), gen_range(-150.0, -80.0)),
                life: 1.0,
                max_life: gen_range(0.5, 1.5),
                size: gen_range(8.0, 25.0),
            }
        }
    }

    fn alive(&self) -> bool {
        self.life > 0.0
    }
}

pub struct FireScene {
    particles: Vec<Particle>,
    embers: Vec<Particle>,
    additive_mat: Option<Material>,
    time: f32,
}

impl FireScene {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            embers: Vec::with_capacity(MAX_EMBERS),
            additive_mat: None,
            time: 0.0,
        }
    }

    fn ensure_material(&mut self) {
        if self.additive_mat.is_some() {
            return;
        }
        let pipeline_params = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::One,
            )),
            ..Default::default()
        };
        self.additive_mat = Some(
            load_material(
                ShaderSource::Glsl {
                    vertex: shaders::VERTEX,
                    fragment: shaders::PASSTHROUGH_FRAG,
                },
                MaterialParams {
                    pipeline_params,
                    ..Default::default()
                },
            )
            .unwrap(),
        );
    }
}

impl Scene for FireScene {
    fn init(&mut self) {
        self.particles.clear();
        self.embers.clear();
        self.time = 0.0;
        self.ensure_material();
    }

    fn update(&mut self, t: f32, dt: f32) {
        self.time = t;
        let cx = screen_width() / 2.0;
        let cy = screen_height() * 0.85;

        // Spawn new particles
        for _ in 0..SPAWN_RATE {
            if self.particles.len() < MAX_PARTICLES {
                self.particles.push(Particle::spawn(cx, cy, false));
            }
        }
        // Spawn embers
        if gen_range(0.0, 1.0) < 0.3 && self.embers.len() < MAX_EMBERS {
            self.embers.push(Particle::spawn(cx, cy, true));
        }

        // Update particles
        for p in &mut self.particles {
            p.pos += p.vel * dt;
            p.vel.y -= 20.0 * dt; // buoyancy
            p.vel.x += (t * 3.0 + p.pos.y * 0.01).sin() * 50.0 * dt; // shimmer
            p.life -= dt / p.max_life;
        }
        self.particles.retain(|p| p.alive());

        // Update embers
        for p in &mut self.embers {
            p.pos += p.vel * dt;
            p.vel.x += (t * 5.0 + p.pos.y * 0.02).sin() * 80.0 * dt;
            p.life -= dt / p.max_life;
        }
        self.embers.retain(|p| p.alive());
    }

    fn draw(&self) {
        let pal = &palette::FIRE;
        clear_background(pal.background);

        if let Some(mat) = &self.additive_mat {
            gl_use_material(mat);
        }

        for p in &self.particles {
            let life = p.life.clamp(0.0, 1.0);
            let color = hsl_to_rgb(life * 0.1, 1.0, 0.2 + life * 0.5);
            let alpha = life;
            let c = Color::new(color.r, color.g, color.b, alpha);
            draw_circle(p.pos.x, p.pos.y, p.size * life, c);
        }

        for p in &self.embers {
            let life = p.life.clamp(0.0, 1.0);
            let color = hsl_to_rgb(0.08, 1.0, 0.4 + life * 0.4);
            let c = Color::new(color.r, color.g, color.b, life * 0.8);
            draw_circle(p.pos.x, p.pos.y, p.size * life, c);
        }

        gl_use_default_material();
    }

    fn name(&self) -> &str {
        "Fire Particles"
    }
}
