use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::palette;
use crate::scene::Scene;
use crate::shaders;

const NUM_SEEDS: usize = 30;

pub struct VoronoiScene {
    seeds: Vec<Vec2>,
    velocities: Vec<Vec2>,
    colors: Vec<Vec4>,
    material: Option<Material>,
    time: f32,
}

impl VoronoiScene {
    pub fn new() -> Self {
        let pal = &palette::VORONOI;
        let sw = 1280.0_f32;
        let sh = 720.0_f32;

        let seeds: Vec<Vec2> = (0..NUM_SEEDS)
            .map(|_| vec2(gen_range(0.0, sw), gen_range(0.0, sh)))
            .collect();

        let velocities: Vec<Vec2> = (0..NUM_SEEDS)
            .map(|_| vec2(gen_range(-80.0, 80.0), gen_range(-80.0, 80.0)))
            .collect();

        let colors: Vec<Vec4> = (0..NUM_SEEDS)
            .map(|i| {
                let c = pal.colors[i % pal.colors.len()];
                vec4(c.r, c.g, c.b, c.a)
            })
            .collect();

        Self {
            seeds,
            velocities,
            colors,
            material: None,
            time: 0.0,
        }
    }

    fn ensure_material(&mut self) {
        if self.material.is_some() {
            return;
        }
        self.material = Some(
            load_material(
                ShaderSource::Glsl {
                    vertex: shaders::VERTEX,
                    fragment: shaders::VORONOI_FRAG,
                },
                MaterialParams {
                    uniforms: vec![
                        UniformDesc::array(
                            UniformDesc::new("seeds", UniformType::Float4),
                            15,
                        ),
                        UniformDesc::array(
                            UniformDesc::new("seed_colors", UniformType::Float4),
                            30,
                        ),
                        UniformDesc::new("resolution", UniformType::Float2),
                    ],
                    ..Default::default()
                },
            )
            .unwrap(),
        );
    }
}

impl Scene for VoronoiScene {
    fn init(&mut self) {
        self.time = 0.0;
        let sw = screen_width();
        let sh = screen_height();
        for seed in &mut self.seeds {
            *seed = vec2(gen_range(0.0, sw), gen_range(0.0, sh));
        }
        self.ensure_material();
    }

    fn update(&mut self, t: f32, dt: f32) {
        self.time = t;
        let sw = screen_width();
        let sh = screen_height();

        for i in 0..NUM_SEEDS {
            self.seeds[i] += self.velocities[i] * dt;

            // Bounce off edges
            if self.seeds[i].x < 0.0 || self.seeds[i].x > sw {
                self.velocities[i].x *= -1.0;
                self.seeds[i].x = self.seeds[i].x.clamp(0.0, sw);
            }
            if self.seeds[i].y < 0.0 || self.seeds[i].y > sh {
                self.velocities[i].y *= -1.0;
                self.seeds[i].y = self.seeds[i].y.clamp(0.0, sh);
            }
        }
    }

    fn draw(&self) {
        clear_background(palette::VORONOI.background);

        if let Some(mat) = &self.material {
            // Pack 30 seeds into 15 vec4s
            let mut packed_seeds = [Vec4::ZERO; 15];
            for (i, slot) in packed_seeds.iter_mut().enumerate() {
                let s1 = if i * 2 < NUM_SEEDS {
                    self.seeds[i * 2]
                } else {
                    Vec2::ZERO
                };
                let s2 = if i * 2 + 1 < NUM_SEEDS {
                    self.seeds[i * 2 + 1]
                } else {
                    Vec2::ZERO
                };
                *slot = vec4(s1.x, s1.y, s2.x, s2.y);
            }

            // Pad colors to 30
            let mut color_arr = [Vec4::ZERO; 30];
            for (i, c) in self.colors.iter().enumerate() {
                color_arr[i] = *c;
            }

            mat.set_uniform_array("seeds", &packed_seeds);
            mat.set_uniform_array("seed_colors", &color_arr);
            mat.set_uniform("resolution", vec2(screen_width(), screen_height()));

            gl_use_material(mat);
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
            gl_use_default_material();
        }
    }

    fn name(&self) -> &str {
        "Voronoi Shatter"
    }
}
