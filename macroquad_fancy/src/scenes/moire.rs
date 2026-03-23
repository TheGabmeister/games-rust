use macroquad::prelude::*;

use crate::palette;
use crate::scene::Scene;

const RING_COUNT: usize = 60;
const SPACING: f32 = 12.0;

pub struct MoireScene {
    time: f32,
}

impl MoireScene {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }
}

impl Scene for MoireScene {
    fn init(&mut self) {
        self.time = 0.0;
    }

    fn update(&mut self, t: f32, _dt: f32) {
        self.time = t;
    }

    fn draw(&self) {
        let pal = &palette::MOIRE;
        clear_background(pal.background);

        let sw = screen_width();
        let sh = screen_height();
        let t = self.time;

        // Center A orbits
        let cx_a = sw / 2.0 + 100.0 * (t * 0.3).cos();
        let cy_a = sh / 2.0 + 80.0 * (t * 0.4).sin();

        // Center B orbits
        let cx_b = sw / 2.0 + 90.0 * (t * 0.35 + 1.0).cos();
        let cy_b = sh / 2.0 + 70.0 * (t * 0.45 + 2.0).sin();

        let color_a = pal.colors[0];
        let color_b = pal.colors[1];

        for i in 1..=RING_COUNT {
            let r = SPACING * i as f32;
            draw_circle_lines(cx_a, cy_a, r, 1.5, color_a);
            draw_circle_lines(cx_b, cy_b, r, 1.5, color_b);
        }
    }

    fn name(&self) -> &str {
        "Moire Patterns"
    }
}
