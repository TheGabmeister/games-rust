use hecs::*;

use crate::components::*;

/// Tick down Lifetime for every entity that has one (projectiles).
pub fn system_tick_lifetime(world: &mut World, query: &mut PreparedQuery<&mut Lifetime>, dt: f32) {
    for lt in query.query_mut(world) {
        lt.0 -= dt;
    }
}

/// Queue despawn for entities whose Lifetime has expired.
pub fn system_remove_expired(world: &World, cmd: &mut CommandBuffer) {
    for (e, _) in world
        .query::<(Entity, &Lifetime)>()
        .iter()
        .filter(|(_, lt)| lt.0 <= 0.0)
    {
        cmd.despawn(e);
    }
}
