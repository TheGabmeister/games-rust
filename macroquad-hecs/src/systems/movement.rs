use macroquad::prelude::*;
use hecs::*;
use ::rand::RngExt;

use crate::components::*;

/// Set a random Velocity each frame for every NPC with a Speed component.
/// Player has no Speed, so it is excluded automatically.
pub fn system_wander_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Velocity, &Speed)>,
) {
    let mut rng = ::rand::rng();
    for (vel, speed) in query.query_mut(world) {
        vel.0 = vec2(
            rng.random_range(-speed.0..speed.0),
            rng.random_range(-speed.0..speed.0),
        );
    }
}

/// Integrate Velocity into Position for every entity that has both.
pub fn system_integrate_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Position, &Velocity)>,
) {
    let dt = get_frame_time();
    for (pos, vel) in query.query_mut(world) {
        pos.0 += vel.0 * dt;
    }
}
