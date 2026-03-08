use hecs::*;
use macroquad::prelude::get_frame_time;

use crate::components::*;

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
