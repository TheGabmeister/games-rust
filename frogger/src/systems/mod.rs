mod frog;
mod physics;
mod river;
mod road;
mod scoring;

pub use frog::{system_death_anim, system_hop_anim, system_input, system_respawn_delay};
pub use physics::{system_move_entities, system_wrap};
pub use river::{system_platform_carry, system_riding, system_turtle_dive};
pub use road::system_vehicle_collision;
pub use scoring::{system_fly, system_home_check, system_timer};

use hecs::{Entity, World};
use macroquad::prelude::Vec2;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GamePhase, GameResources};
use crate::spawner;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Find the single frog entity (used by render.rs and all sub-systems).
pub fn find_frog(world: &World) -> Option<Entity> {
    world.query::<(Entity, &Frog)>().iter().next().map(|(e, _)| e)
}

/// Shrunk AABB overlap test. Margin makes road/river collisions feel fair.
pub(super) fn aabb(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    let m = 4.0;
    ax + m < bx + bw && ax + aw - m > bx &&
    ay + m < by + bh && ay + ah - m > by
}

/// Begin the frog's death sequence.
pub fn trigger_death(world: &mut World, res: &mut GameResources, frog_entity: Entity) {
    if world.get::<&DeathAnim>(frog_entity).is_ok() { return; }
    let _ = world.remove_one::<RidingPlatform>(frog_entity);
    let _ = world.remove_one::<HopAnim>(frog_entity);
    world.insert_one(frog_entity, DeathAnim(DEATH_ANIM_SECS)).unwrap();
    res.phase = GamePhase::PlayerDead;
}

/// Reset the frog to the starting cell (called after home reached or death).
pub(super) fn respawn_frog(world: &mut World) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    let x = OFFSET_X + FROG_START_COL as f32 * TILE + (TILE - FROG_W) * 0.5;
    let y = OFFSET_Y + FROG_START_ROW as f32 * TILE + (TILE - FROG_H) * 0.5;

    world.get::<&mut Position>(frog_entity).unwrap().0 = Vec2::new(x, y);
    {
        let mut cell = world.get::<&mut FrogCell>(frog_entity).unwrap();
        cell.col = FROG_START_COL;
        cell.row = FROG_START_ROW;
    }
    world.get::<&mut BestRow>(frog_entity).unwrap().0 = FROG_START_ROW;

    let _ = world.remove_one::<RidingPlatform>(frog_entity);
    let _ = world.remove_one::<HopAnim>(frog_entity);

    if let Some(meta) = spawner::find_meta(world) {
        world.get::<&mut LevelTimer>(meta).unwrap().0 = TIMER_SECS;
    }
    world.insert_one(frog_entity, RespawnDelay(RESPAWN_DELAY_SECS)).unwrap();
}
