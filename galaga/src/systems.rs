mod combat;
mod enemy;
mod helpers;
mod input;
mod player;
mod stage;

use hecs::World;

use crate::resources::{GameMode, Resources};

pub fn fixed_update(world: &mut World, resources: &mut Resources, dt: f32) {
    input::read_input(resources);

    if resources.input.start_pressed
        && matches!(resources.flow.mode, GameMode::Attract | GameMode::GameOver)
    {
        stage::start_new_game(world, resources);
    }

    if resources.input.pause_pressed {
        input::toggle_pause(resources);
    }

    if resources.flow.mode == GameMode::Pause {
        return;
    }

    resources.flow.mode_timer += dt;
    resources.ui.message_timer = (resources.ui.message_timer - dt).max(0.0);

    if matches!(resources.flow.mode, GameMode::Attract | GameMode::GameOver) {
        return;
    }

    player::handle_player_respawn(world, resources, dt);

    player::player_motion_and_fire(world, resources, dt);
    enemy::schedule_enemy_dives(world, resources, dt);
    enemy::update_enemy_paths(world, resources, dt);
    enemy::update_capture_beams(world, resources, dt);
    enemy::enemy_fire(world, resources, dt);
    combat::move_projectiles(world, dt);
    combat::detect_collisions(world, resources);
    combat::process_events(world, resources);
    stage::cleanup_entities(world, resources);
    combat::flush_spawn_queue(world, resources);
    stage::update_progression(world, resources, dt);

    if resources.score.score > resources.hi_score.value {
        resources.hi_score.value = resources.score.score;
    }
}
