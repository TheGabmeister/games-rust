use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::palette;
use crate::scene::Scene;

const NUM_STARS: usize = 800;
const FOCAL_LENGTH: f32 = 200.0;

struct Star {
    x: f32,
    y: f32,
    z: f32,
}

impl Star {
    fn random() -> Self {
        Self {
            x: gen_range(-1.0, 1.0),
            y: gen_range(-1.0, 1.0),
            z: gen_range(0.01, 1.0),
        }
    }
}

pub struct StarfieldScene {
    stars: Vec<Star>,
    speed: f32,
}

impl StarfieldScene {
    pub fn new() -> Self {
        Self {
            stars: (0..NUM_STARS).map(|_| Star::random()).collect(),
            speed: 0.1,
        }
    }

    fn project(&self, star: &Star) -> (f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        let sx = sw / 2.0 + (star.x / star.z) * FOCAL_LENGTH;
        let sy = sh / 2.0 + (star.y / star.z) * FOCAL_LENGTH;
        (sx, sy)
    }
}

impl Scene for StarfieldScene {
    fn init(&mut self) {
        for star in &mut self.stars {
            *star = Star::random();
        }
        self.speed = 0.1;
    }

    fn update(&mut self, t: f32, dt: f32) {
        // Speed oscillates: slow -> fast -> slow over 10 seconds
        self.speed = 0.1 + 0.7 * (0.5 - 0.5 * (t * std::f32::consts::PI / 5.0).cos());

        for star in &mut self.stars {
            star.z -= self.speed * dt;
            if star.z <= 0.01 {
                *star = Star::random();
                star.z = 1.0;
            }
        }
    }

    fn draw(&self) {
        let pal = &palette::STARFIELD;
        clear_background(pal.background);

        for star in &self.stars {
            let (sx, sy) = self.project(star);

            // Trail: project a point slightly further back
            let trail_z = (star.z + 0.05 * self.speed).min(1.0);
            let sw = screen_width();
            let sh = screen_height();
            let tx = sw / 2.0 + (star.x / trail_z) * FOCAL_LENGTH;
            let ty = sh / 2.0 + (star.y / trail_z) * FOCAL_LENGTH;

            let brightness = 1.0 - star.z;
            let thickness = brightness * 3.0;

            // Interpolate from blue-white (far) to pure white (near)
            let color = Color::new(
                0.7 + 0.3 * brightness,
                0.8 + 0.2 * brightness,
                1.0,
                brightness.clamp(0.1, 1.0),
            );

            draw_line(tx, ty, sx, sy, thickness.max(0.5), color);
        }
    }

    fn name(&self) -> &str {
        "Starfield Warp"
    }
}
