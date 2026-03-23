use macroquad::prelude::*;
use macroquad::rand::gen_range;

struct Star {
    x: f32,
    y: f32,
    size: f32,
    speed: f32,
    brightness: f32,
}

pub struct Starfield {
    stars: Vec<Star>,
    width: f32,
    height: f32,
}

impl Starfield {
    pub fn new(width: f32, height: f32, count: usize) -> Self {
        let mut stars = Vec::with_capacity(count);
        for _ in 0..count {
            let depth = gen_range(0.0f32, 1.0);
            stars.push(Star {
                x: gen_range(0.0, width),
                y: gen_range(0.0, height),
                size: 0.5 + depth * 1.5,
                speed: 5.0 + depth * 25.0,
                brightness: 0.2 + depth * 0.6,
            });
        }
        Self { stars, width, height }
    }

    pub fn update(&mut self, dt: f32) {
        for star in &mut self.stars {
            star.y += star.speed * dt;
            if star.y > self.height {
                star.y -= self.height;
                star.x = gen_range(0.0, self.width);
            }
        }
    }

    pub fn draw(&self) {
        for star in &self.stars {
            let b = star.brightness;
            draw_circle(star.x, star.y, star.size, Color::new(b, b, (b + 0.1).min(1.0), 1.0));
        }
    }
}
