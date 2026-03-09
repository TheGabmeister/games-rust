use crate::constants::*;
use crate::terrain::Terrain;
use crate::world::Camera;
use macroquad::prelude::*;

#[derive(Clone)]
pub enum AstronautState {
    OnGround,
    BeingCarried { _by_enemy_id: u32 },
    Falling { vel_y: f32 },
    Safe, // caught and deposited, or safely on ground after fall
}

pub struct Astronaut {
    pub pos: Vec2,
    pub state: AstronautState,
    pub alive: bool,
    pub id: u32,
}

impl Astronaut {
    pub fn new(pos: Vec2, id: u32) -> Self {
        Astronaut {
            pos,
            state: AstronautState::OnGround,
            alive: true,
            id,
        }
    }

    pub fn update(&mut self, dt: f32, terrain: &Terrain) {
        match &mut self.state {
            AstronautState::Falling { vel_y } => {
                *vel_y += ASTRONAUT_GRAVITY * dt;
                self.pos.y += *vel_y * dt;

                let ground_y = terrain.surface_y(self.pos.x) - ASTRONAUT_HALF_H;
                if self.pos.y >= ground_y {
                    self.pos.y = ground_y;
                    // Hit the ground — transition to Safe (survived fall)
                    self.state = AstronautState::OnGround;
                }
            }
            AstronautState::BeingCarried { .. } => {
                // Position is set by the lander each frame
            }
            AstronautState::OnGround | AstronautState::Safe => {}
        }
    }

    pub fn aabb(&self) -> Rect {
        Rect::new(
            self.pos.x - ASTRONAUT_HALF_W,
            self.pos.y - ASTRONAUT_HALF_H,
            ASTRONAUT_HALF_W * 2.0,
            ASTRONAUT_HALF_H * 2.0,
        )
    }

    pub fn is_catchable(&self) -> bool {
        matches!(self.state, AstronautState::Falling { .. })
    }

    pub fn draw(&self, camera: &Camera) {
        if !self.alive {
            return;
        }

        let sx = camera.world_to_screen_x(self.pos.x);
        let sy = camera.world_to_screen_y(self.pos.y);

        let color = match &self.state {
            AstronautState::BeingCarried { .. } => Color::new(1.0, 1.0, 0.3, 1.0), // yellow when carried
            AstronautState::Falling { .. } => Color::new(0.3, 1.0, 0.5, 1.0), // bright when falling
            _ => Color::new(0.2, 0.9, 0.2, 1.0),
        };

        // Head
        draw_circle(sx, sy - 5.0, 3.5, color);
        // Body
        draw_rectangle(sx - 2.5, sy - 2.0, 5.0, 8.0, color);

        // Arms out when falling
        if matches!(self.state, AstronautState::Falling { .. }) {
            draw_line(sx - 6.0, sy + 1.0, sx + 6.0, sy + 1.0, 2.0, color);
        }
    }
}

/// Spawn astronauts spread across the terrain surface.
pub fn spawn_astronauts(terrain: &Terrain, count: usize, next_id: &mut u32) -> Vec<Astronaut> {
    let spacing = WORLD_WIDTH / count as f32;
    (0..count)
        .map(|i| {
            let wx = spacing * i as f32 + spacing * 0.5;
            let wy = terrain.surface_y(wx) - ASTRONAUT_HALF_H;
            let id = *next_id;
            *next_id += 1;
            Astronaut::new(Vec2::new(wx, wy), id)
        })
        .collect()
}
