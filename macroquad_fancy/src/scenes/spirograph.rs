use macroquad::prelude::*;

use crate::palette;
use crate::scene::Scene;

struct CurveParams {
    big_r: f32,
    small_r: f32,
    pen_d: f32,
    phase: f32,
    color: Color,
    speed: f32,
}

impl CurveParams {
    fn point_at(&self, t: f32) -> Vec2 {
        let diff = self.big_r - self.small_r;
        let ratio = diff / self.small_r;
        let t = t + self.phase;
        vec2(
            diff * t.cos() + self.pen_d * (t * ratio).cos(),
            diff * t.sin() - self.pen_d * (t * ratio).sin(),
        )
    }
}

pub struct SpirographScene {
    curves: Vec<CurveParams>,
    max_t: f32,
}

impl SpirographScene {
    pub fn new() -> Self {
        let pal = &palette::SPIROGRAPH;
        Self {
            curves: vec![
                CurveParams {
                    big_r: 200.0,
                    small_r: 73.0,
                    pen_d: 60.0,
                    phase: 0.0,
                    color: pal.colors[0],
                    speed: 2.5,
                },
                CurveParams {
                    big_r: 180.0,
                    small_r: 47.0,
                    pen_d: 80.0,
                    phase: std::f32::consts::FRAC_PI_3,
                    color: pal.colors[1],
                    speed: 3.0,
                },
                CurveParams {
                    big_r: 160.0,
                    small_r: 89.0,
                    pen_d: 40.0,
                    phase: std::f32::consts::FRAC_PI_3 * 2.0,
                    color: pal.colors[2],
                    speed: 2.0,
                },
                CurveParams {
                    big_r: 140.0,
                    small_r: 61.0,
                    pen_d: 70.0,
                    phase: std::f32::consts::PI,
                    color: pal.colors[3],
                    speed: 2.8,
                },
            ],
            max_t: 0.0,
        }
    }
}

impl Scene for SpirographScene {
    fn init(&mut self) {
        self.max_t = 0.0;
    }

    fn update(&mut self, t: f32, _dt: f32) {
        // Grow the parameter over scene time
        self.max_t = t * 3.0; // ~30 radians over 10s
    }

    fn draw(&self) {
        let pal = &palette::SPIROGRAPH;
        clear_background(pal.background);

        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let step = 0.02;
        let fade_window = 15.0;

        for curve in &self.curves {
            let curve_max_t = self.max_t * curve.speed / 2.5;
            if curve_max_t < step {
                continue;
            }

            let mut t = 0.0_f32;
            let mut prev = curve.point_at(t);

            while t < curve_max_t {
                t += step;
                let curr = curve.point_at(t);

                // Alpha fades for older segments
                let age = curve_max_t - t;
                let alpha = if age > fade_window {
                    0.0
                } else {
                    1.0 - age / fade_window
                };

                if alpha > 0.01 {
                    let c = Color::new(curve.color.r, curve.color.g, curve.color.b, alpha);
                    draw_line(cx + prev.x, cy + prev.y, cx + curr.x, cy + curr.y, 2.0, c);
                }

                prev = curr;
            }
        }
    }

    fn name(&self) -> &str {
        "Spirograph"
    }
}
