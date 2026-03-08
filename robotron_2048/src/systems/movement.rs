use hecs::*;
use macroquad::prelude::{screen_height, screen_width};

use crate::components::*;

/// Integrate Velocity into Position for every entity that has both.
pub fn system_integrate_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Position, &Velocity)>,
    dt: f32,
) {
    for (pos, vel) in query.query_mut(world) {
        pos.0 += vel.0 * dt;
    }
}

/// Clamp every non-projectile entity to screen boundaries using its collider size.
/// Prevents enemies and the player from being pushed or steered off-screen.
pub fn system_clamp_to_arena(world: &mut World) {
    for (pos, collider) in
        &mut world.query::<Without<(&mut Position, &Collider), &Projectile>>()
    {
        let (half_w, half_h) = match collider {
            Collider::Circle { radius } => (*radius, *radius),
            Collider::Box { half_extents } => (half_extents.x, half_extents.y),
        };
        pos.0.x = pos.0.x.clamp(half_w, screen_width() - half_w);
        pos.0.y = pos.0.y.clamp(half_h, screen_height() - half_h);
    }
}
