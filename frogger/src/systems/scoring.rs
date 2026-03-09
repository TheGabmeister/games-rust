/// Scoring and progression: home arrival, countdown timer, fly bonus lifetime.
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::resources::{GamePhase, GameResources};
use crate::spawner;

use super::{find_frog, respawn_frog, trigger_death};

// ── 9. Home check ─────────────────────────────────────────────────────────────

pub fn system_home_check(world: &mut World, res: &mut GameResources) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_ok() { return; }

    let frog_row = world.get::<&FrogCell>(frog_entity).unwrap().row;
    if frog_row != ROW_HOMES { return; }

    let frog_col = world.get::<&FrogCell>(frog_entity).unwrap().col;
    let home_match = HOME_COLS.iter().enumerate().find(|&(_, &c)| c == frog_col);

    let Some((home_idx, _)) = home_match else {
        // Landed between homes (on the hedge).
        trigger_death(world, res, frog_entity);
        return;
    };

    // Find the matching home entity.
    let home_info: Option<(Entity, bool)> = world
        .query::<(Entity, &Home)>()
        .iter()
        .find(|(_, h)| h.idx == home_idx)
        .map(|(e, h)| (e, h.filled));

    match home_info {
        Some((_, true)) => {
            // Already occupied — treat as death.
            trigger_death(world, res, frog_entity);
        }
        Some((home_ent, false)) => {
            world.get::<&mut Home>(home_ent).unwrap().filled = true;

            let meta = match spawner::find_meta(world) { Some(e) => e, None => return };
            let remaining_secs = world.get::<&LevelTimer>(meta).unwrap().0.max(0.0) as i32;

            // Collect any fly bonus in this home.
            let fly_ents: Vec<Entity> = world
                .query::<(Entity, &Fly)>()
                .iter()
                .filter(|(_, f)| f.home_idx == home_idx)
                .map(|(e, _)| e)
                .collect();
            let has_fly = !fly_ents.is_empty();
            for fe in fly_ents { let _ = world.despawn(fe); }

            let bonus = SCORE_HOME
                + remaining_secs * SCORE_TIME_MULT
                + if has_fly { SCORE_FLY } else { 0 };
            world.get::<&mut Score>(meta).unwrap().0 += bonus;
            world.get::<&mut HomesProgress>(meta).unwrap().0[home_idx] = true;
            world.get::<&mut LevelTimer>(meta).unwrap().0 = TIMER_SECS;

            let all_done = world.get::<&HomesProgress>(meta).unwrap().0.iter().all(|&f| f);
            if all_done {
                world.get::<&mut Score>(meta).unwrap().0 += SCORE_LEVEL;
                res.phase = GamePhase::LevelComplete;
            } else {
                respawn_frog(world);
                res.phase = GamePhase::Playing;
            }
        }
        None => {
            trigger_death(world, res, frog_entity);
        }
    }
}

// ── 10. Countdown timer ───────────────────────────────────────────────────────

pub fn system_timer(world: &mut World, res: &mut GameResources, dt: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_ok()    { return; }
    if world.get::<&RespawnDelay>(frog_entity).is_ok() { return; }

    let meta = match spawner::find_meta(world) { Some(e) => e, None => return };
    let expired = {
        let mut t = world.get::<&mut LevelTimer>(meta).unwrap();
        t.0 -= dt;
        if t.0 <= 0.0 { t.0 = 0.0; true } else { false }
    }; // RefMut dropped here before trigger_death borrows world mutably
    if expired {
        trigger_death(world, res, frog_entity);
    }
}

// ── 11. Fly lifetime ──────────────────────────────────────────────────────────

pub fn system_fly(world: &mut World, dt: f32) {
    // Pass 1: collect all fly entities (drops QueryBorrow).
    let fly_ents: Vec<Entity> = world
        .query::<(Entity, &Fly)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    // Pass 2: tick timers individually and mark expired.
    let mut to_despawn = Vec::new();
    for e in fly_ents {
        if let Ok(mut fly) = world.get::<&mut Fly>(e) {
            fly.timer -= dt;
            if fly.timer <= 0.0 {
                to_despawn.push(e);
            }
        }
    }
    for e in to_despawn { let _ = world.despawn(e); }
}
