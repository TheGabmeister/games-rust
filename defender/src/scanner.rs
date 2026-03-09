use macroquad::prelude::*;
use crate::constants::*;
use crate::world::Camera;
use crate::terrain::Terrain;
use crate::player::Player;
use crate::enemies::Enemy;
use crate::astronauts::{Astronaut, AstronautState};

pub struct Scanner;

impl Scanner {
    pub fn draw(
        camera: &Camera,
        player: &Player,
        enemies: &[Enemy],
        astronauts: &[Astronaut],
        terrain: &Terrain,
    ) {
        let sw = screen_width();
        let sh = SCANNER_HEIGHT;
        let scale = sw / WORLD_WIDTH;

        // Background
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.08, 1.0));

        // Terrain: sample every 2px column
        let terrain_color = Color::new(0.1, 0.45, 0.1, 1.0);
        let mut x_px = 0usize;
        while x_px < sw as usize {
            let world_x = x_px as f32 / scale;
            let h_frac = terrain.height_at(world_x) / TERRAIN_MAX_HEIGHT;
            let scanner_y = sh - h_frac * sh * 0.65;
            draw_rectangle(x_px as f32, scanner_y, 2.0, sh - scanner_y, terrain_color);
            x_px += 2;
        }

        // Viewport indicator (white outline)
        let vp_world_w = sw; // viewport covers screen_width world units
        let vp_scanner_w = vp_world_w * scale;
        let vp_scanner_x = camera.x * scale;
        // Clamp draw so it doesn't bleed off the scanner at wrap boundary
        draw_rectangle_lines(vp_scanner_x % sw, 1.0, vp_scanner_w, sh - 2.0, 1.0,
                             Color::new(0.6, 0.6, 0.6, 0.5));

        // Enemy dots (red)
        for e in enemies.iter().filter(|e| e.alive) {
            let ex = (e.pos.x * scale) % sw;
            let ey = sh * 0.35;
            draw_rectangle(ex - 1.5, ey - 1.5, 3.0, 3.0, Color::new(1.0, 0.15, 0.15, 1.0));
        }

        // Astronaut dots (green)
        for a in astronauts.iter().filter(|a| a.alive) {
            if matches!(a.state, AstronautState::Safe) { continue; }
            let ax = (a.pos.x * scale) % sw;
            let ay = sh * 0.78;
            draw_rectangle(ax - 1.5, ay - 1.5, 3.0, 3.0, Color::new(0.15, 1.0, 0.15, 1.0));
        }

        // Player (white, larger)
        if player.alive {
            let px = (player.pos.x * scale) % sw;
            let py = sh * 0.5;
            draw_rectangle(px - 3.0, py - 3.0, 6.0, 6.0, WHITE);
        }

        // Separator line
        draw_line(0.0, sh, sw, sh, 1.5, Color::new(0.3, 0.3, 0.3, 1.0));
    }
}
