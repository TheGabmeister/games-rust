/// Road mechanics: vehicle collision detection.
use hecs::World;

use crate::components::*;
use crate::constants::*;
use crate::resources::GameResources;

use super::{aabb, find_frog, trigger_death};

// ── 8. Vehicle collision ──────────────────────────────────────────────────────

pub fn system_vehicle_collision(world: &mut World, res: &mut GameResources) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_ok() { return; }

    let frog_row = world.get::<&FrogCell>(frog_entity).unwrap().row;
    if frog_row < ROW_ROAD_TOP || frog_row > ROW_ROAD_BOT { return; }

    let (fx, fy, fw, fh) = {
        let pos  = world.get::<&Position>(frog_entity).unwrap();
        let size = world.get::<&Size>(frog_entity).unwrap();
        (pos.0.x, pos.0.y, size.0.x, size.0.y)
    };

    // Snapshot vehicle rects (drops QueryBorrow before potential mutation).
    let vehicles: Vec<(f32, f32, f32, f32)> = world
        .query::<(&Position, &Size, &Vehicle)>()
        .iter()
        .map(|(p, s, _)| (p.0.x, p.0.y, s.0.x, s.0.y))
        .collect();

    for (vx, vy, vw, vh) in vehicles {
        if aabb(fx, fy, fw, fh, vx, vy, vw, vh) {
            trigger_death(world, res, frog_entity);
            return;
        }
    }
}
