/// River mechanics: turtle diving, platform riding detection, platform carry.
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::resources::GameResources;

use super::{aabb, find_frog, trigger_death};

// ── 5. Turtle dive state machine ──────────────────────────────────────────────

pub fn system_turtle_dive(world: &mut World, dt: f32) {
    for (dive, color) in world.query_mut::<(&mut TurtleDive, &mut DrawColor)>() {
        dive.timer -= dt;
        if dive.timer <= 0.0 {
            let next = TurtleDive::next_phase(dive.phase);
            dive.timer = TurtleDive::phase_duration(next);
            dive.phase = next;
        }

        color.0.a = match dive.phase {
            DivePhase::Surface   => 1.0,
            DivePhase::Diving    => {
                (dive.timer / TurtleDive::phase_duration(DivePhase::Diving)).max(0.3)
            }
            DivePhase::Submerged => 0.15,
            DivePhase::Rising    => {
                let total = TurtleDive::phase_duration(DivePhase::Rising);
                (1.0 - dive.timer / total).max(0.15)
            }
        };
    }
}

// ── 6. Platform riding detection ─────────────────────────────────────────────

pub fn system_riding(world: &mut World, res: &mut GameResources) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_ok() { return; }
    // Skip while the frog is mid-hop — it hasn't landed yet.
    if world.get::<&HopAnim>(frog_entity).is_ok() { return; }

    let frog_row = world.get::<&FrogCell>(frog_entity).unwrap().row;
    if frog_row < ROW_RIVER_TOP || frog_row > ROW_RIVER_BOT {
        let _ = world.remove_one::<RidingPlatform>(frog_entity);
        return;
    }

    // Pass 1: collect frog rect (drops borrow before platform query).
    let (fx, fy, fw, fh) = {
        let pos  = world.get::<&Position>(frog_entity).unwrap();
        let size = world.get::<&Size>(frog_entity).unwrap();
        (pos.0.x, pos.0.y, size.0.x, size.0.y)
    };

    // Pass 2: snapshot all platform rects + velocities.
    let platforms: Vec<(Entity, f32, f32, f32, f32, f32)> = world
        .query::<(Entity, &Position, &Size, &Velocity, &Platform)>()
        .iter()
        .map(|(e, pos, sz, vel, _)| (e, pos.0.x, pos.0.y, sz.0.x, sz.0.y, vel.0))
        .collect();

    // Pass 3: find the first rideable platform that overlaps the frog.
    let mut found: Option<(Entity, f32)> = None;
    for (pe, px, py, pw, ph, pv) in &platforms {
        let rideable = match world.get::<&TurtleDive>(*pe) {
            Ok(dive) => dive.is_rideable(),
            Err(_)   => true, // logs / alligators always rideable
        };
        if rideable && aabb(fx, fy, fw, fh, *px, *py, *pw, *ph) {
            found = Some((*pe, *pv));
            break;
        }
    }

    match found {
        None => trigger_death(world, res, frog_entity),
        Some((pe, _)) => {
            if let Ok(mut riding) = world.get::<&mut RidingPlatform>(frog_entity) {
                riding.0 = pe;
            } else {
                world.insert_one(frog_entity, RidingPlatform(pe)).unwrap();
            }
        }
    }
}

// ── 7. Platform carry ─────────────────────────────────────────────────────────

pub fn system_platform_carry(world: &mut World, res: &mut GameResources, dt: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_ok() { return; }
    if world.get::<&HopAnim>(frog_entity).is_ok()  { return; }

    let platform_entity = match world.get::<&RidingPlatform>(frog_entity) {
        Ok(r)  => r.0,
        Err(_) => return,
    };
    let vel_x = match world.get::<&Velocity>(platform_entity) {
        Ok(v)  => v.0,
        Err(_) => return,
    };

    world.get::<&mut Position>(frog_entity).unwrap().0.x += vel_x * dt;

    // Carried off-screen → drown.
    let frog_x = world.get::<&Position>(frog_entity).unwrap().0.x;
    if frog_x + FROG_W < OFFSET_X - TILE || frog_x > OFFSET_X + COLS as f32 * TILE + TILE {
        trigger_death(world, res, frog_entity);
    }
}
