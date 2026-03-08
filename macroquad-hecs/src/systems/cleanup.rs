use macroquad::prelude::get_frame_time;
use hecs::*;

use crate::components::*;

/// Tick down Lifetime for every entity that has one (projectiles).
pub fn system_tick_lifetime(world: &mut World, query: &mut PreparedQuery<&mut Lifetime>) {
    let dt = get_frame_time();
    for lt in query.query_mut(world) {
        lt.0 -= dt;
    }
}

/// Despawn entities whose Lifetime has expired.
pub fn system_remove_expired(world: &mut World) {
    let to_remove: Vec<Entity> = world
        .query::<(Entity, &Lifetime)>()
        .iter()
        .filter(|(_, lt)| lt.0 <= 0.0)
        .map(|(e, _)| e)
        .collect();
    for e in to_remove {
        let _ = world.despawn(e);
    }
}
