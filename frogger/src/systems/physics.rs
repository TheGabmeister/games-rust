/// Pure entity movement: velocity integration and screen-edge wrapping.
use hecs::World;

use crate::components::*;

// ── 3. Move all entities with velocity ───────────────────────────────────────

pub fn system_move_entities(world: &mut World, dt: f32) {
    for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.0.x += vel.0 * dt;
    }
}

// ── 4. Wrap entities at screen edges ─────────────────────────────────────────

pub fn system_wrap(world: &mut World) {
    for (pos, vel, size, bounds) in
        world.query_mut::<(&mut Position, &Velocity, &Size, &WrapBounds)>()
    {
        if vel.0 > 0.0 && pos.0.x > bounds.x_max {
            pos.0.x = bounds.x_min - size.0.x;
        } else if vel.0 < 0.0 && pos.0.x + size.0.x < bounds.x_min {
            pos.0.x = bounds.x_max;
        }
    }
}
