use crate::constants::*;
use crate::world::Camera;
use macroquad::prelude::*;

pub struct Terrain {
    /// Height of terrain surface from the bottom of the play area, per segment.
    pub heights: Vec<f32>,
    pub segment_width: f32,
}

impl Terrain {
    pub fn generate(seed: u32) -> Self {
        let segment_width = WORLD_WIDTH / TERRAIN_SEGMENTS as f32;
        let mut heights = Vec::with_capacity(TERRAIN_SEGMENTS);

        // Seeded random walk with smoothing
        let mut rng_state = seed;
        let mut next_rand = move || -> f32 {
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 17;
            rng_state ^= rng_state << 5;
            (rng_state as f32) / (u32::MAX as f32)
        };

        let mid = (TERRAIN_MIN_HEIGHT + TERRAIN_MAX_HEIGHT) / 2.0;
        let mut h = mid;
        for _ in 0..TERRAIN_SEGMENTS {
            h += (next_rand() - 0.5) * 30.0;
            h = h.clamp(TERRAIN_MIN_HEIGHT, TERRAIN_MAX_HEIGHT);
            heights.push(h);
        }

        // Smooth pass (box filter, 3-tap)
        let len = heights.len();
        let mut smoothed = heights.clone();
        for i in 0..len {
            let prev = heights[(i + len - 1) % len];
            let next = heights[(i + 1) % len];
            smoothed[i] = (heights[i] * 2.0 + prev + next) / 4.0;
        }

        Terrain {
            heights: smoothed,
            segment_width,
        }
    }

    /// World-space y coordinate of the terrain surface at a given world x.
    pub fn surface_y(&self, world_x: f32) -> f32 {
        let seg_idx = self.seg_at(world_x);
        WORLD_HEIGHT - self.heights[seg_idx]
    }

    /// Height value (from bottom) at the given world x.
    pub fn height_at(&self, world_x: f32) -> f32 {
        self.heights[self.seg_at(world_x)]
    }

    fn seg_at(&self, world_x: f32) -> usize {
        let idx = (world_x / self.segment_width) as usize;
        idx % TERRAIN_SEGMENTS
    }

    pub fn draw(&self, camera: &Camera) {
        let sw = screen_width();
        let screen_bottom = screen_height();

        let segments_visible = (sw / self.segment_width).ceil() as usize + 2;
        let start_seg = (camera.x / self.segment_width) as usize;

        for i in 0..=segments_visible {
            let seg_idx = (start_seg + i) % TERRAIN_SEGMENTS;
            let next_idx = (seg_idx + 1) % TERRAIN_SEGMENTS;

            let world_x0 = (start_seg + i) as f32 * self.segment_width;
            let world_x1 = world_x0 + self.segment_width;

            let sx0 = camera.world_to_screen_x(world_x0);
            let sx1 = camera.world_to_screen_x(world_x1);

            let h0 = self.heights[seg_idx];
            let h1 = self.heights[next_idx];

            // Screen y of terrain surface (lower screen_y = higher up)
            let sy0 = screen_bottom - h0;
            let sy1 = screen_bottom - h1;

            let terrain_fill = Color::new(0.05, 0.28, 0.05, 1.0);
            let ridge_color = Color::new(0.1, 0.75, 0.15, 1.0);

            // Two triangles forming trapezoid down to screen bottom
            draw_triangle(
                Vec2::new(sx0, sy0),
                Vec2::new(sx1, sy1),
                Vec2::new(sx1, screen_bottom),
                terrain_fill,
            );
            draw_triangle(
                Vec2::new(sx0, sy0),
                Vec2::new(sx1, screen_bottom),
                Vec2::new(sx0, screen_bottom),
                terrain_fill,
            );

            // Bright ridge line
            draw_line(sx0, sy0, sx1, sy1, 2.0, ridge_color);
        }
    }
}
