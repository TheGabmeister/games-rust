use hecs::*;

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
