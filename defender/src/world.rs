use macroquad::prelude::*;
use crate::constants::*;

pub struct Camera {
    /// x-position in world space of the LEFT edge of the viewport
    pub x: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { x: 0.0 }
    }

    /// Center the viewport on the player's world x.
    pub fn follow(&mut self, player_world_x: f32) {
        self.x = (player_world_x - screen_width() / 2.0).rem_euclid(WORLD_WIDTH);
    }

    /// Convert world x → screen x, handling wrap-around.
    pub fn world_to_screen_x(&self, world_x: f32) -> f32 {
        let mut dx = world_x - self.x;
        // Bring dx into the range [-WORLD_WIDTH/2, WORLD_WIDTH/2]
        if dx < -WORLD_WIDTH / 2.0 {
            dx += WORLD_WIDTH;
        }
        if dx > WORLD_WIDTH / 2.0 {
            dx -= WORLD_WIDTH;
        }
        dx
    }

    /// Convert world y → screen y (offset by scanner strip height).
    pub fn world_to_screen_y(&self, world_y: f32) -> f32 {
        world_y + SCANNER_HEIGHT
    }

    /// Convert screen x → world x.
    pub fn screen_to_world_x(&self, screen_x: f32) -> f32 {
        (self.x + screen_x).rem_euclid(WORLD_WIDTH)
    }

    /// Returns true if the world-space point (with given half-width) is visible on screen.
    pub fn is_visible(&self, world_x: f32, half_w: f32) -> bool {
        let sx = self.world_to_screen_x(world_x);
        sx + half_w > 0.0 && sx - half_w < screen_width()
    }
}

/// Wrap-aware normalized direction vector from `from` to `to`.
pub fn wrap_direction(from: Vec2, to: Vec2) -> Vec2 {
    let mut dx = to.x - from.x;
    if dx < -WORLD_WIDTH / 2.0 {
        dx += WORLD_WIDTH;
    }
    if dx > WORLD_WIDTH / 2.0 {
        dx -= WORLD_WIDTH;
    }
    let dy = to.y - from.y;
    let v = Vec2::new(dx, dy);
    if v.length_squared() < 0.0001 {
        Vec2::ZERO
    } else {
        v.normalize()
    }
}

/// Wrap-aware distance squared between two world positions.
pub fn wrap_dist_sq(a: Vec2, b: Vec2) -> f32 {
    let mut dx = b.x - a.x;
    if dx < -WORLD_WIDTH / 2.0 {
        dx += WORLD_WIDTH;
    }
    if dx > WORLD_WIDTH / 2.0 {
        dx -= WORLD_WIDTH;
    }
    let dy = b.y - a.y;
    dx * dx + dy * dy
}
