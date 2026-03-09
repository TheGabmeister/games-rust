use crate::constants::*;
use crate::terrain::Terrain;
use crate::world::Camera;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum FacingDir {
    Left,
    Right,
}

pub struct FireCommand {
    pub pos: Vec2,
    pub vel: Vec2,
}

#[derive(Default)]
pub struct PlayerCommands {
    pub fire: Option<FireCommand>,
    pub smart_bomb: bool,
    pub hyperspace: bool,
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: FacingDir,
    pub alive: bool,
    pub smart_bombs: u32,
    pub fire_cooldown: f32,
    pub invincible_timer: f32,
    /// True when player is actively thrusting (for engine glow)
    pub thrusting: bool,
    /// Astronaut id that player is currently carrying (caught mid-air)
    pub carried_astronaut: Option<u32>,
}

impl Player {
    pub fn new(world_x: f32, world_y: f32) -> Self {
        Player {
            pos: Vec2::new(world_x, world_y),
            vel: Vec2::ZERO,
            facing: FacingDir::Right,
            alive: true,
            smart_bombs: SMART_BOMBS_PER_LIFE,
            fire_cooldown: 0.0,
            invincible_timer: PLAYER_INVINCIBLE_TIME,
            thrusting: false,
            carried_astronaut: None,
        }
    }

    /// Update player state and return player commands for this frame.
    pub fn update(&mut self, dt: f32, terrain: &Terrain) -> PlayerCommands {
        let mut commands = PlayerCommands::default();

        if !self.alive {
            return commands;
        }

        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }

        // --- Movement ---
        let mut vx = 0.0f32;
        let mut vy = 0.0f32;

        let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        let up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        let down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);

        if left {
            vx = -PLAYER_SPEED;
            self.facing = FacingDir::Left;
        } else if right {
            vx = PLAYER_SPEED;
            self.facing = FacingDir::Right;
        }

        if up {
            vy = -PLAYER_VERT_SPEED;
        } else if down {
            vy = PLAYER_VERT_SPEED;
        }

        self.thrusting = vx != 0.0 || vy != 0.0;
        self.vel = Vec2::new(vx, vy);

        // --- Integrate ---
        self.pos.x = (self.pos.x + self.vel.x * dt).rem_euclid(WORLD_WIDTH);
        self.pos.y += self.vel.y * dt;

        // --- Clamp to world bounds ---
        let terrain_y = terrain.surface_y(self.pos.x);
        // Clamp above terrain
        self.pos.y = self.pos.y.min(terrain_y - PLAYER_HALF_H - 2.0);
        // Clamp below scanner strip
        self.pos.y = self.pos.y.max(SCANNER_HEIGHT + PLAYER_HALF_H + 2.0);

        // --- Fire ---
        let fire = is_key_down(KeyCode::Z) || is_key_down(KeyCode::LeftControl);
        if fire && self.fire_cooldown <= 0.0 {
            self.fire_cooldown = PLAYER_FIRE_RATE;
            let vbx = match self.facing {
                FacingDir::Right => PLAYER_BULLET_SPEED,
                FacingDir::Left => -PLAYER_BULLET_SPEED,
            };
            // Spawn bullet at ship nose
            let nose_offset = match self.facing {
                FacingDir::Right => PLAYER_HALF_W + 2.0,
                FacingDir::Left => -(PLAYER_HALF_W + 2.0),
            };
            commands.fire = Some(FireCommand {
                pos: Vec2::new(
                    (self.pos.x + nose_offset).rem_euclid(WORLD_WIDTH),
                    self.pos.y,
                ),
                vel: Vec2::new(vbx, 0.0),
            });
        }

        // --- Smart bomb ---
        if is_key_pressed(KeyCode::X) && self.smart_bombs > 0 {
            self.smart_bombs -= 1;
            commands.smart_bomb = true;
        }

        // --- Hyperspace ---
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::LeftShift) {
            commands.hyperspace = true;
        }

        commands
    }

    pub fn aabb(&self) -> Rect {
        Rect::new(
            self.pos.x - PLAYER_HALF_W,
            self.pos.y - PLAYER_HALF_H,
            PLAYER_HALF_W * 2.0,
            PLAYER_HALF_H * 2.0,
        )
    }

    pub fn draw(&self, camera: &Camera) {
        if !self.alive {
            return;
        }

        let sx = camera.world_to_screen_x(self.pos.x);
        let sy = camera.world_to_screen_y(self.pos.y);

        // Blink when invincible
        if self.invincible_timer > 0.0 {
            let blink = ((self.invincible_timer * 10.0) as u32).is_multiple_of(2);
            if blink {
                return;
            }
        }

        draw_ship(sx, sy, self.facing, self.thrusting);
    }
}

fn draw_ship(sx: f32, sy: f32, facing: FacingDir, thrusting: bool) {
    let (tip, rear_top, rear_bot) = match facing {
        FacingDir::Right => (
            Vec2::new(sx + PLAYER_HALF_W, sy),
            Vec2::new(sx - PLAYER_HALF_W, sy - PLAYER_HALF_H),
            Vec2::new(sx - PLAYER_HALF_W, sy + PLAYER_HALF_H),
        ),
        FacingDir::Left => (
            Vec2::new(sx - PLAYER_HALF_W, sy),
            Vec2::new(sx + PLAYER_HALF_W, sy - PLAYER_HALF_H),
            Vec2::new(sx + PLAYER_HALF_W, sy + PLAYER_HALF_H),
        ),
    };

    draw_triangle(tip, rear_top, rear_bot, crate::constants::CYAN);

    // Engine exhaust
    if thrusting {
        let exhaust_x = match facing {
            FacingDir::Right => sx - PLAYER_HALF_W - 6.0,
            FacingDir::Left => sx + PLAYER_HALF_W + 6.0,
        };
        draw_circle(exhaust_x, sy, 4.0, ORANGE);
    }

    // Cockpit
    let cockpit_x = match facing {
        FacingDir::Right => sx + 4.0,
        FacingDir::Left => sx - 4.0,
    };
    draw_circle(cockpit_x, sy, 3.5, Color::new(0.4, 1.0, 1.0, 1.0));
}

/// Draw a small ship icon for the HUD lives display.
pub fn draw_mini_ship(sx: f32, sy: f32) {
    draw_triangle(
        Vec2::new(sx + 8.0, sy),
        Vec2::new(sx - 8.0, sy - 5.0),
        Vec2::new(sx - 8.0, sy + 5.0),
        crate::constants::CYAN,
    );
}
