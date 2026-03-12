use hecs::World;

use crate::resources::DespawnQueue;

/// Applies all queued despawns once per frame.
pub fn system_apply_despawns(world: &mut World, despawns: &mut DespawnQueue) {
    for entity in despawns.drain() {
        let _ = world.despawn(entity);
    }
}
