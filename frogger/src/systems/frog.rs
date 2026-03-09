/// Frog lifecycle: input handling, hop animation, death animation, respawn delay.
use hecs::World;
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GamePhase, GameResources};
use crate::spawner;

use super::{find_frog, respawn_frog};

// ── 1. Input ──────────────────────────────────────────────────────────────────

pub fn system_input(world: &mut World, res: &GameResources) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };

    let blocked = world.get::<&HopAnim>(frog_entity).is_ok()
        || world.get::<&DeathAnim>(frog_entity).is_ok()
        || world.get::<&RespawnDelay>(frog_entity).is_ok()
        || res.phase != GamePhase::Playing;
    if blocked { return; }

    let up    = is_key_pressed(KeyCode::Up)    || is_key_pressed(KeyCode::W);
    let down  = is_key_pressed(KeyCode::Down)  || is_key_pressed(KeyCode::S);
    let left  = is_key_pressed(KeyCode::Left)  || is_key_pressed(KeyCode::A);
    let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

    let (dcol, drow) = if up { (0, -1) } else if down { (0, 1) }
                       else if left { (-1, 0) } else if right { (1, 0) }
                       else { return };

    let (col, row, px, py) = {
        let cell = world.get::<&FrogCell>(frog_entity).unwrap();
        let pos  = world.get::<&Position>(frog_entity).unwrap();
        (cell.col, cell.row, pos.0.x, pos.0.y)
    };

    let new_col = (col + dcol).clamp(0, COLS - 1);
    let new_row = (row + drow).clamp(ROW_HOMES, ROW_START);

    let tx = OFFSET_X + new_col as f32 * TILE + (TILE - FROG_W) * 0.5;
    let ty = OFFSET_Y + new_row as f32 * TILE + (TILE - FROG_H) * 0.5;

    {
        let mut cell = world.get::<&mut FrogCell>(frog_entity).unwrap();
        cell.col = new_col;
        cell.row = new_row;
    }

    // Award points only for genuinely new forward rows.
    if new_row < row {
        let best = world.get::<&BestRow>(frog_entity).unwrap().0;
        if new_row < best {
            world.get::<&mut BestRow>(frog_entity).unwrap().0 = new_row;
            if let Some(meta) = spawner::find_meta(world) {
                world.get::<&mut Score>(meta).unwrap().0 += SCORE_HOP;
            }
        }
    }

    world.insert_one(frog_entity, HopAnim {
        t:    0.0,
        from: Vec2::new(px, py),
        to:   Vec2::new(tx, ty),
    }).unwrap();
}

// ── 2. Hop animation ──────────────────────────────────────────────────────────

pub fn system_hop_anim(world: &mut World, dt: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&HopAnim>(frog_entity).is_err() { return; }

    let (new_t, to) = {
        let mut anim = world.get::<&mut HopAnim>(frog_entity).unwrap();
        anim.t += dt / HOP_DURATION;
        (anim.t, anim.to)
    };

    if new_t >= 1.0 {
        world.get::<&mut Position>(frog_entity).unwrap().0 = to;
        world.remove_one::<HopAnim>(frog_entity).unwrap();
    } else {
        let (from, to2) = {
            let anim = world.get::<&HopAnim>(frog_entity).unwrap();
            (anim.from, anim.to)
        };
        let lerped = from.lerp(to2, new_t);
        let arc = -(new_t * std::f32::consts::PI).sin() * 8.0;
        world.get::<&mut Position>(frog_entity).unwrap().0 =
            Vec2::new(lerped.x, lerped.y + arc);
    }
}

// ── 12. Death animation ───────────────────────────────────────────────────────

pub fn system_death_anim(world: &mut World, res: &mut GameResources, dt: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&DeathAnim>(frog_entity).is_err() { return; }

    let remaining = {
        let mut anim = world.get::<&mut DeathAnim>(frog_entity).unwrap();
        anim.0 -= dt;
        anim.0
    };

    if remaining <= 0.0 {
        world.remove_one::<DeathAnim>(frog_entity).unwrap();
        let meta = match spawner::find_meta(world) { Some(e) => e, None => return };
        world.get::<&mut Lives>(meta).unwrap().0 -= 1;

        if world.get::<&Lives>(meta).unwrap().0 <= 0 {
            res.phase = GamePhase::GameOver;
        } else {
            respawn_frog(world);
            res.phase = GamePhase::Playing;
        }
    }
}

// ── 13. Respawn delay ─────────────────────────────────────────────────────────

pub fn system_respawn_delay(world: &mut World, dt: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };
    if world.get::<&RespawnDelay>(frog_entity).is_err() { return; }

    let remaining = {
        let mut delay = world.get::<&mut RespawnDelay>(frog_entity).unwrap();
        delay.0 -= dt;
        delay.0
    };

    if remaining <= 0.0 {
        let _ = world.remove_one::<RespawnDelay>(frog_entity);
    }
}
