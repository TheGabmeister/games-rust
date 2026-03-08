#![windows_subsystem = "windows"] // Suppress the console window on Windows release builds.
#![cfg_attr(debug_assertions, allow(unused))] // Suppress unused-code warnings in debug builds.

use hecs::{CommandBuffer, PreparedQuery, World};
use macroquad::prelude::*;

mod collision;
mod components;
mod resources;
mod systems;

use components::{Lifetime, Position, Velocity, WaveClearTarget};
use resources::{GameState, InputState, Resources, SoundId};
use systems::*;

const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_FRAME_TIME: f32 = 0.25;

struct SimulationCaches {
    integrate_query: PreparedQuery<(&'static mut Position, &'static Velocity)>,
    lifetime_query: PreparedQuery<&'static mut Lifetime>,
    commands: CommandBuffer,
    accumulator: f32,
}

impl SimulationCaches {
    fn new() -> Self {
        Self {
            integrate_query: PreparedQuery::default(),
            lifetime_query: PreparedQuery::default(),
            commands: CommandBuffer::new(),
            accumulator: 0.0,
        }
    }

    fn reset_timestep(&mut self) {
        self.accumulator = 0.0;
    }
}

#[macroquad::main("Robotron 2084")]
async fn main() {
    let mut res = Resources::load().await;
    let mut world = World::new();
    let mut sim = SimulationCaches::new();

    loop {
        system_capture_input(&mut res.input);
        let input = res.input;
        if input.debug_toggle_pressed {
            res.debug_enabled = !res.debug_enabled;
        }

        let frame_dt = get_frame_time().min(MAX_FRAME_TIME);

        match res.state {
            GameState::MainMenu => {
                sim.reset_timestep();
                update_main_menu(&mut world, &mut res, input);
            }
            GameState::Playing => {
                sim.accumulator += frame_dt;
                let mut step_input = input;

                while sim.accumulator >= FIXED_DT {
                    update_playing_step(&mut world, &mut res, &mut sim, step_input, FIXED_DT);
                    if res.state != GameState::Playing {
                        sim.reset_timestep();
                        break;
                    }
                    sim.accumulator -= FIXED_DT;
                    step_input = step_input.fixed_step_continuation();
                }

                system_audio(&mut res);
            }
            GameState::Paused => {
                sim.reset_timestep();
                update_paused(&mut res, input);
            }
            GameState::GameOver => {
                sim.reset_timestep();
                update_game_over(&mut res, input);
            }
        }

        clear_background(BLACK);
        match res.state {
            GameState::MainMenu => draw_main_menu(),
            GameState::Playing => draw_playing(&world, &res),
            GameState::Paused => draw_paused(&world, &res),
            GameState::GameOver => draw_game_over(&res),
        }

        next_frame().await;
    }
}

fn update_main_menu(world: &mut World, res: &mut Resources, input: InputState) {
    if !input.confirm_pressed {
        return;
    }

    world.clear();
    spawn_player(world);
    res.score = 0;
    res.player_died = false;
    res.wave_director.reset();
    system_wave_director(world, &mut res.wave_director);
    res.state = GameState::Playing;
    start_music(res);
}

fn update_playing_step(
    world: &mut World,
    res: &mut Resources,
    sim: &mut SimulationCaches,
    input: InputState,
    dt: f32,
) {
    if input.cancel_pressed {
        res.state = GameState::Paused;
        return;
    }

    system_wave_director(world, &mut res.wave_director);
    system_enemy_ai(world, dt);
    system_enemy_attack(world, dt);
    system_enemy_spawn(world, dt);
    system_enemy_maturation(world, dt);
    system_player_move(world, input);
    system_player_shoot(world, input, res);
    system_integrate_velocity(world, &mut sim.integrate_query, dt);
    system_clamp_to_arena(world);
    system_projectile_collision(world, &mut sim.commands, res);
    system_tick_lifetime(world, &mut sim.lifetime_query, dt);
    system_remove_expired(world, &mut sim.commands);

    // Apply deferred structural edits before any logic that reads world state.
    sim.commands.run_on(world);

    if system_player_contact_damage(world) {
        res.player_died = true;
        res.state = GameState::GameOver;
        stop_music(res);
        res.queue_sound(SoundId::Lose);
        return;
    }

    let wave_cleared = world.query::<&WaveClearTarget>().iter().next().is_none();
    if wave_cleared {
        res.wave_director.queue_next_wave();
        system_wave_director(world, &mut res.wave_director);
    }
}

fn update_paused(res: &mut Resources, input: InputState) {
    if input.resume_pressed {
        res.state = GameState::Playing;
    }
    if input.cancel_pressed {
        stop_music(res);
        res.state = GameState::MainMenu;
    }
}

fn update_game_over(res: &mut Resources, input: InputState) {
    if input.confirm_pressed {
        res.state = GameState::MainMenu;
    }
}

fn draw_main_menu() {
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
}

fn draw_playing(world: &World, res: &Resources) {
    system_draw(world, res);
    if res.debug_enabled {
        system_draw_colliders(world);
        draw_text("DEBUG COLLIDERS [F1]", 10.0, 60.0, 16.0, LIME);
    }
    draw_text(&format!("Score: {}", res.score), 10.0, 20.0, 20.0, WHITE);
    draw_text(
        &format!("Wave: {}", res.wave_director.wave_number()),
        10.0,
        40.0,
        20.0,
        WHITE,
    );
    draw_text(
        "[WASD] move  [LMB] shoot  [Esc] pause  [F1] debug",
        10.0,
        60.0,
        16.0,
        DARKGRAY,
    );
}

fn draw_paused(world: &World, res: &Resources) {
    system_draw(world, res);
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
    draw_text(
        "[Space] resume  [Esc] menu",
        cx - 140.0,
        cy + 22.0,
        22.0,
        GRAY,
    );
    if res.debug_enabled {
        system_draw_colliders(world);
        draw_text("DEBUG COLLIDERS [F1]", 10.0, 20.0, 16.0, LIME);
    }
}

fn draw_game_over(res: &Resources) {
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    let (headline, color) = if res.player_died {
        ("YOU WERE DESTROYED", RED)
    } else {
        ("RUN COMPLETE", GREEN)
    };
    draw_text(headline, cx - 195.0, cy - 40.0, 38.0, color);
    draw_text(
        &format!("Score: {}", res.score),
        cx - 60.0,
        cy + 10.0,
        30.0,
        WHITE,
    );
    draw_text("[Enter] main menu", cx - 110.0, cy + 52.0, 22.0, GRAY);
}
