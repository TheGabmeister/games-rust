mod components;
mod constants;
mod render;
mod resources;
mod spawner;
mod systems;

use hecs::World;
use macroquad::prelude::*;

use constants::*;
use render::*;
use resources::{GamePhase, GameResources};
use spawner::*;
use systems::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Frogger".to_string(),
        window_width: WINDOW_W as i32,
        window_height: WINDOW_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new();
    let mut res   = GameResources::new();
    let mut time  = 0.0f32;
    let mut level_complete_timer = 0.0f32;

    loop {
        let dt = get_frame_time().min(0.05);
        time += dt;

        match res.phase {
            // ── Title screen ──────────────────────────────────────────────────
            GamePhase::TitleScreen => {
                render_title();
                if any_key_pressed() {
                    spawn_all(&mut world, res.speed_scale, START_LIVES, 0, 1);
                    res.phase = GamePhase::Playing;
                }
            }

            // ── Playing ───────────────────────────────────────────────────────
            GamePhase::Playing => {
                system_input(&mut world, &res);
                system_hop_anim(&mut world, dt);
                system_move_entities(&mut world, dt);
                system_wrap(&mut world);
                system_turtle_dive(&mut world, dt);
                system_riding(&mut world, &mut res);
                if res.phase == GamePhase::Playing {
                    system_platform_carry(&mut world, &mut res, dt);
                }
                if res.phase == GamePhase::Playing {
                    system_vehicle_collision(&mut world, &mut res);
                }
                if res.phase == GamePhase::Playing {
                    system_home_check(&mut world, &mut res);
                }
                if res.phase == GamePhase::Playing {
                    system_timer(&mut world, &mut res, dt);
                }
                system_fly(&mut world, dt);
                system_respawn_delay(&mut world, dt);
                render_all(&world, &res, time);
            }

            // ── Death animation ───────────────────────────────────────────────
            GamePhase::PlayerDead => {
                system_move_entities(&mut world, dt);
                system_wrap(&mut world);
                system_turtle_dive(&mut world, dt);
                system_fly(&mut world, dt);
                system_death_anim(&mut world, &mut res, dt);
                render_all(&world, &res, time);
            }

            // ── Level complete ────────────────────────────────────────────────
            GamePhase::LevelComplete => {
                level_complete_timer += dt;
                render_all(&world, &res, time);

                if level_complete_timer >= 2.5 {
                    level_complete_timer = 0.0;

                    let meta = find_meta(&world).expect("meta entity missing");
                    let lives = world.get::<&components::Lives>(meta).unwrap().0;
                    let score = world.get::<&components::Score>(meta).unwrap().0;
                    let level = world.get::<&components::Level>(meta).unwrap().0;

                    res.advance_speed();
                    world.clear();
                    spawn_all(&mut world, res.speed_scale, lives, score, level + 1);
                    res.phase = GamePhase::Playing;
                }
            }

            // ── Game over ─────────────────────────────────────────────────────
            GamePhase::GameOver => {
                render_all(&world, &res, time);
                if any_key_pressed() {
                    world.clear();
                    res = GameResources::new();
                    res.phase = GamePhase::TitleScreen;
                }
            }
        }

        next_frame().await;
    }
}

fn any_key_pressed() -> bool {
    [
        KeyCode::Space, KeyCode::Enter,
        KeyCode::Up,    KeyCode::Down,
        KeyCode::Left,  KeyCode::Right,
        KeyCode::W,     KeyCode::A,
        KeyCode::S,     KeyCode::D,
        KeyCode::Escape,
    ]
    .iter()
    .any(|&k| is_key_pressed(k))
}
