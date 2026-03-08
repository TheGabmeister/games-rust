#![windows_subsystem = "windows"] // Suppress the console window on Windows release builds.
#![cfg_attr(debug_assertions, allow(unused))] // Suppress unused-code warnings in debug builds.

use hecs::{CommandBuffer, PreparedQuery};
use macroquad::prelude::*;

mod collision;
mod components;
mod resources;
mod systems;

use components::{EnemyKind, Lifetime, Position, Velocity};
use resources::{GameState, Resources, SoundId};
use systems::*;

#[macroquad::main("Robotron 2084")]
async fn main() {
    let mut res = Resources::load().await;
    let mut world = hecs::World::new();

    // Cached PreparedQuery instances: created once, reused every frame.
    let mut integrate_query = PreparedQuery::<(&mut Position, &Velocity)>::default();
    let mut lifetime_query = PreparedQuery::<&mut Lifetime>::default();

    // Reused deferred command buffer for structural world edits.
    let mut commands = CommandBuffer::new();

    loop {
        // Capture all input exactly once per frame into a singleton resource.
        system_capture_input(&mut res.input);
        let input = res.input;
        if input.debug_toggle_pressed {
            res.debug_enabled = !res.debug_enabled;
        }

        match res.state {
            GameState::MainMenu => {
                clear_background(BLACK);
                let cx = screen_width() / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("ROBOTRON 2084", cx - 150.0, cy - 50.0, 52.0, WHITE);
                draw_text("Press [Enter] to start", cx - 130.0, cy + 8.0, 26.0, GRAY);
                draw_text(
                    "[WASD] move  [LMB] shoot  [F1] debug",
                    cx - 190.0,
                    cy + 40.0,
                    20.0,
                    DARKGRAY,
                );

                if input.confirm_pressed {
                    world.clear();
                    batch_spawn_enemies(&mut world, 50);
                    spawn_player(&mut world);
                    res.score = 0;
                    res.state = GameState::Playing;
                    start_music(&res);
                }
            }

            GameState::Playing => {
                // --- update ---
                system_enemy_ai(&mut world);
                system_enemy_attack(&mut world);
                system_enemy_spawn(&mut world);
                system_player_move(&mut world, input);
                system_player_shoot(&mut world, input, &mut res);
                system_integrate_velocity(&mut world, &mut integrate_query);
                system_projectile_collision(&mut world, &mut commands, &mut res);
                system_tick_lifetime(&mut world, &mut lifetime_query);
                system_remove_expired(&world, &mut commands);

                // Apply deferred structural changes before logic checks/draw.
                commands.run_on(&mut world);

                system_audio(&mut res);

                // Wave complete when all enemies are gone.
                if world.query::<&EnemyKind>().iter().count() == 0 {
                    res.state = GameState::GameOver;
                    stop_music(&res);
                    res.queue_sound(SoundId::Lose);
                    system_audio(&mut res);
                }

                if input.cancel_pressed {
                    res.state = GameState::Paused;
                }

                // --- draw ---
                clear_background(BLACK);
                system_draw(&world, &res);
                if res.debug_enabled {
                    system_draw_colliders(&world);
                    draw_text("DEBUG COLLIDERS [F1]", 10.0, 60.0, 16.0, LIME);
                }
                draw_text(&format!("Score: {}", res.score), 10.0, 20.0, 20.0, WHITE);
                draw_text(
                    "[WASD] move  [LMB] shoot  [Esc] pause  [F1] debug",
                    10.0,
                    40.0,
                    16.0,
                    DARKGRAY,
                );
            }

            GameState::Paused => {
                // Draw the frozen world behind the overlay.
                clear_background(BLACK);
                system_draw(&world, &res);
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.55),
                );

                let cx = screen_width() / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("PAUSED", cx - 80.0, cy - 30.0, 52.0, YELLOW);
                draw_text("[Space] resume  [Esc] menu", cx - 140.0, cy + 22.0, 22.0, GRAY);
                if res.debug_enabled {
                    system_draw_colliders(&world);
                    draw_text("DEBUG COLLIDERS [F1]", 10.0, 20.0, 16.0, LIME);
                }

                if input.resume_pressed {
                    res.state = GameState::Playing;
                }
                if input.cancel_pressed {
                    stop_music(&res);
                    res.state = GameState::MainMenu;
                }
            }

            GameState::GameOver => {
                clear_background(BLACK);
                let cx = screen_width() / 2.0;
                let cy = screen_height() / 2.0;
                draw_text("ALL ENEMIES DEFEATED", cx - 195.0, cy - 40.0, 38.0, GREEN);
                draw_text(&format!("Score: {}", res.score), cx - 60.0, cy + 10.0, 30.0, WHITE);
                draw_text("[Enter] main menu", cx - 110.0, cy + 52.0, 22.0, GRAY);

                if input.confirm_pressed {
                    res.state = GameState::MainMenu;
                }
            }
        }

        next_frame().await;
    }
}
