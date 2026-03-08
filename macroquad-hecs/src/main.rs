#![windows_subsystem = "windows"] // Suppress the console window on Windows release builds.
#![cfg_attr(debug_assertions, allow(unused))] // Suppress unused-code warnings in debug builds.

use macroquad::prelude::*;
use hecs::PreparedQuery;

mod assets;
mod components;
mod resources;
mod systems;

use assets::Assets;
use components::{Position, Velocity, Speed};
use systems::*;

#[macroquad::main("Game")]
async fn main() {
    let assets = Assets::load().await;
    let mut world = hecs::World::new();

    batch_spawn_entities(&mut world, 50);
    spawn_player(&mut world, assets.player_ship);

    // Cache PreparedQuery instances once — avoids archetype re-discovery each frame.
    let mut wander_query    = PreparedQuery::<(&mut Velocity, &Speed)>::default();
    let mut integrate_query = PreparedQuery::<(&mut Position, &Velocity)>::default();

    let mut paused = false;

    loop {
        if is_key_pressed(KeyCode::Space)  { paused = !paused; }
        if is_key_pressed(KeyCode::Escape) { break; }

        if !paused {
            system_player_input(&mut world);
            system_wander_velocity(&mut world, &mut wander_query);
            system_integrate_velocity(&mut world, &mut integrate_query);
            system_fire_at_closest(&mut world);
            system_remove_dead(&mut world);
        }

        clear_background(BLACK);
        system_draw(&world);
        if paused {
            draw_text("PAUSED  [Space] resume  [Esc] quit", 10.0, 20.0, 20.0, YELLOW);
        } else {
            draw_text("[Space] pause  [Esc] quit", 10.0, 20.0, 20.0, GRAY);
        }

        next_frame().await;
    }
}
