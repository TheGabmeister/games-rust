#![windows_subsystem = "windows"] // Suppress the console window on Windows release builds.
#![cfg_attr(debug_assertions, allow(unused))] // Suppress unused-code warnings in debug builds.

use macroquad::prelude::*;
use hecs::PreparedQuery;

mod components;
mod collision;
mod resources;
mod systems;

use components::{Position, Velocity, Speed, Lifetime, Enemy};
use resources::{GameState, Resources, SoundId};
use systems::*;

#[macroquad::main("Robotron 2084")]
async fn main() {
    let mut res  = Resources::load().await;
    let mut world = hecs::World::new();

    // Cached PreparedQuery instances — created once, reused every frame.
    // hecs uses these to skip archetype re-discovery on each call.
    let mut wander_query    = PreparedQuery::<(&mut Velocity, &Speed)>::default();
    let mut integrate_query = PreparedQuery::<(&mut Position, &Velocity)>::default();
    let mut lifetime_query  = PreparedQuery::<&mut Lifetime>::default();

    loop {
        match res.state {

            // ── Main Menu ────────────────────────────────────────────────────
            GameState::MainMenu => {
                clear_background(BLACK);
                let cx = screen_width()  / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("ROBOTRON 2084",             cx - 150.0, cy - 50.0, 52.0, WHITE);
                draw_text("Press [Enter] to start",    cx - 130.0, cy +  8.0, 26.0, GRAY);
                draw_text("[WASD] move  [LMB] shoot",  cx - 130.0, cy + 40.0, 20.0, DARKGRAY);

                if is_key_pressed(KeyCode::Enter) {
                    world.clear();
                    batch_spawn_entities(&mut world, 50);
                    spawn_player(&mut world);
                    res.score = 0;
                    res.state = GameState::Playing;
                    start_music(&res);
                }
            }

            // ── Playing ──────────────────────────────────────────────────────
            GameState::Playing => {
                // --- update ---
                system_player_input(&mut world);
                system_player_shoot(&mut world, &mut res);
                system_wander_velocity(&mut world, &mut wander_query);
                system_integrate_velocity(&mut world, &mut integrate_query);
                system_projectile_collision(&mut world, &mut res);
                system_tick_lifetime(&mut world, &mut lifetime_query);
                system_remove_expired(&mut world);
                system_audio(&mut res);

                // Wave complete when all enemies are gone.
                if world.query::<&Enemy>().iter().count() == 0 {
                    res.state = GameState::GameOver;
                    stop_music(&res);
                    res.queue_sound(SoundId::Lose);
                    system_audio(&mut res);
                }

                if is_key_pressed(KeyCode::Escape) {
                    res.state = GameState::Paused;
                }

                // --- draw ---
                clear_background(BLACK);
                system_draw(&world, &res);
                draw_text(&format!("Score: {}", res.score), 10.0, 20.0, 20.0, WHITE);
                draw_text("[WASD] move  [LMB] shoot  [Esc] pause", 10.0, 40.0, 16.0, DARKGRAY);
            }

            // ── Paused ───────────────────────────────────────────────────────
            GameState::Paused => {
                // Draw the frozen world behind the overlay.
                clear_background(BLACK);
                system_draw(&world, &res);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                               Color::new(0.0, 0.0, 0.0, 0.55));

                let cx = screen_width()  / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("PAUSED",                        cx - 80.0,  cy - 30.0, 52.0, YELLOW);
                draw_text("[Space] resume  [Esc] menu",    cx - 140.0, cy + 22.0, 22.0, GRAY);

                if is_key_pressed(KeyCode::Space)  { res.state = GameState::Playing; }
                if is_key_pressed(KeyCode::Escape) {
                    stop_music(&res);
                    res.state = GameState::MainMenu;
                }
            }

            // ── Game Over ────────────────────────────────────────────────────
            GameState::GameOver => {
                clear_background(BLACK);
                let cx = screen_width()  / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("ALL ENEMIES DEFEATED",          cx - 195.0, cy - 40.0, 38.0, GREEN);
                draw_text(&format!("Score: {}", res.score), cx -  60.0, cy + 10.0, 30.0, WHITE);
                draw_text("[Enter] main menu",              cx - 110.0, cy + 52.0, 22.0, GRAY);

                if is_key_pressed(KeyCode::Enter) {
                    res.state = GameState::MainMenu;
                }
            }
        }

        next_frame().await;
    }
}
