use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GamePhase, GameResources};
use crate::spawner;
use crate::systems::find_frog;

// ── Colours ───────────────────────────────────────────────────────────────────

const COL_ROAD:    Color = Color { r: 0.22, g: 0.22, b: 0.22, a: 1.0 };
const COL_RIVER:   Color = Color { r: 0.05, g: 0.18, b: 0.45, a: 1.0 };
const COL_SAFE:    Color = Color { r: 0.12, g: 0.35, b: 0.12, a: 1.0 };
const COL_HOME_BG: Color = Color { r: 0.05, g: 0.12, b: 0.30, a: 1.0 };
const COL_HOME_OK: Color = Color { r: 0.10, g: 0.60, b: 0.10, a: 1.0 };
const COL_HUD_BG:  Color = Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 };

// ── Background ────────────────────────────────────────────────────────────────

pub fn render_background() {
    clear_background(Color::from_rgba(10, 10, 10, 255));

    let gx = OFFSET_X;
    let gw = COLS as f32 * TILE;

    draw_rectangle(0.0, 0.0, OFFSET_X, WINDOW_H, COL_HUD_BG);
    draw_rectangle(OFFSET_X + gw, 0.0, OFFSET_X, WINDOW_H, COL_HUD_BG);

    for row in 0..ROWS {
        let y = OFFSET_Y + row as f32 * TILE;
        let col = match row {
            0     => COL_HOME_BG,
            1..=5 => COL_RIVER,
            6     => COL_SAFE,
            7..=11 => COL_ROAD,
            _     => COL_SAFE,
        };
        draw_rectangle(gx, y, gw, TILE, col);
    }

    // Road lane dashes.
    for row in ROW_ROAD_TOP..=ROW_ROAD_BOT {
        let y = OFFSET_Y + row as f32 * TILE + TILE - 2.0;
        let mut x = gx;
        while x < gx + gw {
            draw_rectangle(x, y, 20.0, 2.0, Color::from_rgba(80, 80, 80, 200));
            x += 36.0;
        }
    }
}

// ── Home pockets ──────────────────────────────────────────────────────────────

pub fn render_homes(world: &World) {
    // hecs 0.11: query().iter() yields Q::Item — no Entity unless in tuple.
    for (home, pos, size) in world.query::<(&Home, &Position, &Size)>().iter() {
        let x = pos.0.x;
        let y = pos.0.y;
        let w = size.0.x;
        let h = size.0.y;

        if home.filled {
            draw_rectangle(x, y, w, h, COL_HOME_OK);
            let cx = x + w * 0.5;
            let cy = y + h * 0.5;
            draw_circle(cx, cy, 7.0, GREEN);
            draw_circle(cx - 4.0, cy - 6.0, 3.0, GREEN);
            draw_circle(cx + 4.0, cy - 6.0, 3.0, GREEN);
        } else {
            draw_rectangle(x + 4.0, y + 4.0, w - 8.0, h - 8.0,
                           Color::from_rgba(20, 60, 20, 255));
            draw_rectangle_lines(x + 4.0, y + 4.0, w - 8.0, h - 8.0, 2.0,
                                  Color::from_rgba(40, 120, 40, 200));
        }
    }
}

// ── Flies ─────────────────────────────────────────────────────────────────────

pub fn render_flies(world: &World) {
    for fly in world.query::<&Fly>().iter() {
        let col = HOME_COLS[fly.home_idx];
        let cx = OFFSET_X + col as f32 * TILE + TILE * 0.5;
        let cy = OFFSET_Y + ROW_HOMES as f32 * TILE + TILE * 0.5;
        draw_circle(cx, cy, 5.0, YELLOW);
        draw_ellipse(cx - 8.0, cy - 2.0, 6.0, 3.0, Color::from_rgba(255, 255, 200, 180));
        draw_ellipse(cx + 8.0, cy - 2.0, 6.0, 3.0, Color::from_rgba(255, 255, 200, 180));
    }
}

fn draw_ellipse(x: f32, y: f32, rx: f32, ry: f32, color: Color) {
    let steps = 16u32;
    use std::f32::consts::TAU;
    let pts: Vec<Vec2> = (0..=steps)
        .map(|i| {
            let a = i as f32 / steps as f32 * TAU;
            Vec2::new(x + a.cos() * rx, y + a.sin() * ry)
        })
        .collect();
    for i in 0..steps as usize {
        draw_line(pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y, 1.5, color);
    }
}

// ── Platforms (logs, turtles) ─────────────────────────────────────────────────

pub fn render_platforms(world: &World) {
    // Include Entity in query tuple to enable world.get checks.
    let platforms: Vec<(Entity, f32, f32, f32, f32, Color)> = world
        .query::<(Entity, &Position, &Size, &DrawColor, &Platform)>()
        .iter()
        .map(|(e, pos, sz, col, _)| (e, pos.0.x, pos.0.y, sz.0.x, sz.0.y, col.0))
        .collect();

    for (entity, x, y, w, h, base_col) in platforms {
        let is_turtle = world.get::<&TurtleDive>(entity).is_ok();
        let alpha = base_col.a;

        if is_turtle {
            let num_shells = 2usize;
            let shell_w = w / num_shells as f32;
            for i in 0..num_shells {
                let sx = x + i as f32 * shell_w + 2.0;
                let sw = shell_w - 4.0;
                let c = Color { a: alpha, ..base_col };
                draw_rectangle(sx, y, sw, h, c);
                let dark = Color { r: c.r * 0.7, g: c.g * 0.7, b: c.b * 0.7, a: alpha };
                draw_rectangle(sx + sw * 0.3, y + 2.0, sw * 0.4, h - 4.0, dark);
                draw_circle(sx + sw * 0.5, y - 3.0, 3.5,
                            Color { r: 0.3, g: 0.5, b: 0.2, a: alpha });
            }
        } else {
            draw_rectangle(x, y, w, h, base_col);
            let grain = Color { r: base_col.r * 0.8, g: base_col.g * 0.8,
                                b: base_col.b * 0.8, a: alpha };
            let num_grains = (w / 20.0) as i32;
            for i in 0..num_grains {
                let lx = x + (i as f32 + 0.5) * (w / num_grains as f32);
                draw_rectangle(lx, y + 2.0, 2.0, h - 4.0, grain);
            }
            draw_rectangle_lines(x, y, w, h, 1.5, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.4 });
        }
    }
}

// ── Vehicles ──────────────────────────────────────────────────────────────────

pub fn render_vehicles(world: &World) {
    for (pos, size, color, _) in
        world.query::<(&Position, &Size, &DrawColor, &Vehicle)>().iter()
    {
        let x = pos.0.x;
        let y = pos.0.y;
        let w = size.0.x;
        let h = size.0.y;

        draw_rectangle(x, y, w, h, color.0);
        let highlight = Color { r: (color.0.r + 0.3).min(1.0),
                                 g: (color.0.g + 0.3).min(1.0),
                                 b: (color.0.b + 0.3).min(1.0),
                                 a: 0.9 };
        draw_rectangle(x + w * 0.2, y + 2.0, w * 0.6, h * 0.4, highlight);
        let wheel = Color::from_rgba(20, 20, 20, 255);
        draw_rectangle(x + 2.0, y + h - 5.0, 6.0, 4.0, wheel);
        draw_rectangle(x + w - 8.0, y + h - 5.0, 6.0, 4.0, wheel);
        draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(0, 0, 0, 120));
    }
}

// ── Frog ──────────────────────────────────────────────────────────────────────

pub fn render_frog(world: &World, time: f32) {
    let frog_entity = match find_frog(world) { Some(e) => e, None => return };

    let dying   = world.get::<&DeathAnim>(frog_entity).is_ok();
    let waiting = world.get::<&RespawnDelay>(frog_entity).is_ok();
    if waiting { return; }

    let draw_pos = world.get::<&Position>(frog_entity).unwrap().0;
    let w = FROG_W;
    let h = FROG_H;
    let cx = draw_pos.x + w * 0.5;
    let cy = draw_pos.y + h * 0.5;

    let frog_color = if dying {
        let anim_t = world.get::<&DeathAnim>(frog_entity).unwrap().0;
        if (anim_t * 8.0) as i32 % 2 == 0 { RED } else { WHITE }
    } else {
        GREEN
    };

    draw_circle(cx, cy, w * 0.45, frog_color);

    if !dying {
        let eye_c = Color::from_rgba(20, 20, 20, 255);
        draw_circle(cx - w * 0.22, cy - h * 0.25, 4.5, WHITE);
        draw_circle(cx + w * 0.22, cy - h * 0.25, 4.5, WHITE);
        draw_circle(cx - w * 0.22, cy - h * 0.25, 2.5, eye_c);
        draw_circle(cx + w * 0.22, cy - h * 0.25, 2.5, eye_c);

        let leg_col = Color::from_rgba(0, 180, 0, 255);
        let has_hop = world.get::<&HopAnim>(frog_entity).is_ok();
        if has_hop {
            let t = world.get::<&HopAnim>(frog_entity).unwrap().t;
            let spread = (t * std::f32::consts::PI).sin() * 8.0;
            draw_line(cx - 5.0, cy + 6.0, cx - 14.0 - spread, cy + 16.0, 2.5, leg_col);
            draw_line(cx + 5.0, cy + 6.0, cx + 14.0 + spread, cy + 16.0, 2.5, leg_col);
            draw_line(cx - 5.0, cy - 4.0, cx - 10.0, cy - 12.0, 2.0, leg_col);
            draw_line(cx + 5.0, cy - 4.0, cx + 10.0, cy - 12.0, 2.0, leg_col);
        } else {
            let bob = (time * 3.0).sin();
            draw_line(cx - 5.0, cy + 5.0 + bob, cx - 14.0, cy + 14.0 + bob, 2.5, leg_col);
            draw_line(cx + 5.0, cy + 5.0 + bob, cx + 14.0, cy + 14.0 + bob, 2.5, leg_col);
            draw_line(cx - 5.0, cy - 3.0, cx - 12.0, cy - 10.0, 2.0, leg_col);
            draw_line(cx + 5.0, cy - 3.0, cx + 12.0, cy - 10.0, 2.0, leg_col);
        }
    }
}

// ── HUD ───────────────────────────────────────────────────────────────────────

pub fn render_hud(world: &World) {
    let meta = match spawner::find_meta(world) { Some(e) => e, None => return };

    let score = world.get::<&Score>(meta).map(|s| s.0).unwrap_or(0);
    let lives = world.get::<&Lives>(meta).map(|l| l.0).unwrap_or(0);
    let level = world.get::<&Level>(meta).map(|l| l.0).unwrap_or(1);
    let timer = world.get::<&LevelTimer>(meta).map(|t| t.0).unwrap_or(0.0);

    let lx = 8.0;
    let rx = OFFSET_X + COLS as f32 * TILE + 8.0;

    draw_text("SCORE", lx, 40.0, 16.0, LIGHTGRAY);
    draw_text(&format!("{:06}", score), lx, 60.0, 22.0, WHITE);
    draw_text("LEVEL", lx, 100.0, 16.0, LIGHTGRAY);
    draw_text(&format!("{}", level), lx, 120.0, 22.0, WHITE);

    draw_text("LIVES", lx, 160.0, 16.0, LIGHTGRAY);
    for i in 0..lives.max(0) {
        let ix = lx + 8.0 + i as f32 * 24.0;
        let iy = 185.0;
        draw_circle(ix, iy, 8.0, GREEN);
        draw_circle(ix - 4.0, iy - 7.0, 3.0, GREEN);
        draw_circle(ix + 4.0, iy - 7.0, 3.0, GREEN);
    }

    draw_text("TIME", rx, 40.0, 16.0, LIGHTGRAY);
    let bar_h = 200.0;
    let bar_w = 18.0;
    let frac = (timer / TIMER_SECS).clamp(0.0, 1.0);
    draw_rectangle(rx, 55.0, bar_w, bar_h, Color::from_rgba(40, 40, 40, 255));
    let bar_color = if frac > 0.5 { GREEN } else if frac > 0.25 { YELLOW } else { RED };
    draw_rectangle(rx, 55.0 + bar_h * (1.0 - frac), bar_w, bar_h * frac, bar_color);
    draw_rectangle_lines(rx, 55.0, bar_w, bar_h, 1.5, LIGHTGRAY);
    draw_text(&format!("{:.0}", timer), rx, 270.0, 18.0, WHITE);

    draw_text("HOMES", rx, 310.0, 16.0, LIGHTGRAY);
    if let Ok(hp) = world.get::<&HomesProgress>(meta) {
        for (i, &f) in hp.0.iter().enumerate() {
            let px = rx + i as f32 * 14.0;
            draw_circle(px + 6.0, 335.0, 5.0, if f { GREEN } else { DARKGRAY });
        }
    }
}

// ── Overlay screens ───────────────────────────────────────────────────────────

pub fn render_title() {
    clear_background(Color::from_rgba(0, 40, 0, 255));
    let cx = WINDOW_W * 0.5;
    draw_text_centered("F R O G G E R", cx, 160.0, 60.0, YELLOW);
    draw_text_centered("Use Arrow Keys or WASD to move", cx, 250.0, 22.0, WHITE);
    draw_text_centered("Cross the road and river to reach your homes", cx, 285.0, 18.0, LIGHTGRAY);
    draw_text_centered("Land on logs and turtles to cross the river", cx, 315.0, 18.0, LIGHTGRAY);
    draw_text_centered("Avoid vehicles  |  Don't fall in the water", cx, 345.0, 18.0, LIGHTGRAY);
    draw_text_centered("Press any key to start", cx, 420.0, 26.0,
                       if (get_time() * 2.0) as i32 % 2 == 0 { WHITE } else { DARKGRAY });
    draw_circle(cx, 520.0, 22.0, GREEN);
    draw_circle(cx - 10.0, 496.0, 8.0, GREEN);
    draw_circle(cx + 10.0, 496.0, 8.0, GREEN);
    draw_circle(cx - 10.0, 496.0, 4.0, BLACK);
    draw_circle(cx + 10.0, 496.0, 4.0, BLACK);
}

pub fn render_game_over(world: &World) {
    let score = spawner::find_meta(world)
        .and_then(|m| world.get::<&Score>(m).ok().map(|s| s.0))
        .unwrap_or(0);
    draw_rectangle(150.0, 180.0, 500.0, 240.0, Color::from_rgba(0, 0, 0, 200));
    draw_rectangle_lines(150.0, 180.0, 500.0, 240.0, 3.0, RED);
    let cx = WINDOW_W * 0.5;
    draw_text_centered("GAME OVER", cx, 250.0, 56.0, RED);
    draw_text_centered(&format!("SCORE: {}", score), cx, 320.0, 30.0, WHITE);
    draw_text_centered("Press any key to restart", cx, 370.0, 22.0,
                       if (get_time() * 2.0) as i32 % 2 == 0 { WHITE } else { DARKGRAY });
}

pub fn render_level_complete(world: &World) {
    let level = spawner::find_meta(world)
        .and_then(|m| world.get::<&Level>(m).ok().map(|l| l.0))
        .unwrap_or(1);
    draw_rectangle(150.0, 200.0, 500.0, 180.0, Color::from_rgba(0, 40, 0, 220));
    draw_rectangle_lines(150.0, 200.0, 500.0, 180.0, 3.0, GREEN);
    let cx = WINDOW_W * 0.5;
    draw_text_centered("LEVEL COMPLETE!", cx, 265.0, 48.0, YELLOW);
    draw_text_centered(&format!("Level {} cleared!", level - 1), cx, 330.0, 28.0, WHITE);
    draw_text_centered("Get ready...", cx, 360.0, 22.0, LIGHTGRAY);
}

fn draw_text_centered(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, size, color);
}

// ── Master render ─────────────────────────────────────────────────────────────

pub fn render_all(world: &World, res: &GameResources, time: f32) {
    render_background();
    render_homes(world);
    render_flies(world);
    render_platforms(world);
    render_vehicles(world);
    render_frog(world, time);
    render_hud(world);

    match res.phase {
        GamePhase::GameOver      => render_game_over(world),
        GamePhase::LevelComplete => render_level_complete(world),
        _                        => {}
    }
}
