use macroquad::prelude::*;
use macroquad::rand::gen_range;
use macroquad::window::miniquad::*;

use crate::palette;
use crate::scene::Scene;
use crate::shaders;

struct Curtain {
    base_y: f32,
    amp1: f32,
    freq1: f32,
    phase1: f32,
    amp2: f32,
    freq2: f32,
    phase2: f32,
    color: Color,
    alpha: f32,
    height: f32,
    speed1: f32,
    speed2: f32,
}

impl Curtain {
    fn top_y(&self, x: f32, t: f32) -> f32 {
        self.base_y
            + self.amp1 * (x * self.freq1 + t * self.speed1 + self.phase1).sin()
            + self.amp2 * (x * self.freq2 + t * self.speed2 + self.phase2).sin()
    }
}

pub struct AuroraScene {
    curtains: Vec<Curtain>,
    stars: Vec<Vec2>,
    additive_mat: Option<Material>,
    time: f32,
}

impl AuroraScene {
    pub fn new() -> Self {
        let pal = &palette::AURORA;
        let sw = 1280.0_f32; // approximate, will use screen_width() at draw time
        let sh = 720.0_f32;

        let curtains = vec![
            Curtain {
                base_y: sh * 0.15,
                amp1: 40.0, freq1: 0.008, phase1: 0.0,
                amp2: 20.0, freq2: 0.015, phase2: 1.0,
                color: pal.colors[0], alpha: 0.25, height: sh * 0.5,
                speed1: 0.8, speed2: 1.2,
            },
            Curtain {
                base_y: sh * 0.20,
                amp1: 35.0, freq1: 0.01, phase1: 2.0,
                amp2: 25.0, freq2: 0.02, phase2: 3.0,
                color: pal.colors[2], alpha: 0.20, height: sh * 0.45,
                speed1: 0.6, speed2: 1.0,
            },
            Curtain {
                base_y: sh * 0.25,
                amp1: 50.0, freq1: 0.006, phase1: 4.0,
                amp2: 15.0, freq2: 0.018, phase2: 0.5,
                color: pal.colors[4], alpha: 0.22, height: sh * 0.4,
                speed1: 0.9, speed2: 1.4,
            },
            Curtain {
                base_y: sh * 0.18,
                amp1: 30.0, freq1: 0.012, phase1: 1.5,
                amp2: 18.0, freq2: 0.022, phase2: 2.5,
                color: pal.colors[1], alpha: 0.18, height: sh * 0.35,
                speed1: 0.7, speed2: 0.9,
            },
            Curtain {
                base_y: sh * 0.22,
                amp1: 45.0, freq1: 0.007, phase1: 3.0,
                amp2: 22.0, freq2: 0.016, phase2: 4.0,
                color: pal.colors[3], alpha: 0.15, height: sh * 0.42,
                speed1: 1.1, speed2: 1.3,
            },
        ];

        // Random star positions
        let stars: Vec<Vec2> = (0..100)
            .map(|_| vec2(gen_range(0.0, sw), gen_range(0.0, sh * 0.6)))
            .collect();

        Self {
            curtains,
            stars,
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

impl Scene for AuroraScene {
    fn init(&mut self) {
        self.time = 0.0;
        self.ensure_material();
    }

    fn update(&mut self, t: f32, _dt: f32) {
        self.time = t;
    }

    fn draw(&self) {
        let pal = &palette::AURORA;
        clear_background(pal.background);

        let sw = screen_width();

        // Draw background stars
        for &star in &self.stars {
            let twinkle = (self.time * 2.0 + star.x * 0.1).sin() * 0.3 + 0.4;
            draw_circle(star.x, star.y, 1.0, Color::new(1.0, 1.0, 1.0, twinkle));
        }

        // Draw curtains with additive blending
        if let Some(mat) = &self.additive_mat {
            gl_use_material(mat);
        }

        let strip_w = 3.0;
        let gradient_steps = 10;

        for curtain in &self.curtains {
            let mut x = 0.0_f32;
            while x < sw {
                let top = curtain.top_y(x, self.time);
                let seg_h = curtain.height / gradient_steps as f32;

                for step in 0..gradient_steps {
                    let frac = step as f32 / gradient_steps as f32;
                    let alpha = curtain.alpha * (1.0 - frac);
                    let y = top + seg_h * step as f32;
                    let c = Color::new(curtain.color.r, curtain.color.g, curtain.color.b, alpha);
                    draw_rectangle(x, y, strip_w, seg_h, c);
                }

                // Occasional bright vertical ray
                let ray_hash = ((x * 7.31 + self.time * 0.5).sin() * 43758.55).fract();
                if ray_hash > 0.97 {
                    let ray_alpha = curtain.alpha * 0.6;
                    let c = Color::new(curtain.color.r, curtain.color.g, curtain.color.b, ray_alpha);
                    draw_line(x, top, x, top + curtain.height * 0.6, 1.5, c);
                }

                x += strip_w;
            }
        }

        gl_use_default_material();
    }

    fn name(&self) -> &str {
        "Aurora Borealis"
    }
}
