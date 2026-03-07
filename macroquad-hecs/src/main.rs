mod assets;
mod collision;
mod debug;
mod ecs;
mod game;
mod state_machine;
mod states;

use assets::{AssetManager, DEFAULT_ASSET_MANIFEST};
use debug::{debug_enabled, draw_debug_overlay, toggle_debug};
use game::GameData;
use macroquad::prelude::*;
use state_machine::StateMachine;
use states::{AppState, PausedState, PlayingState};

#[macroquad::main("Macroquad + hecs Template")]
async fn main() {
    let (assets, startup_warning) = match AssetManager::from_manifest(DEFAULT_ASSET_MANIFEST).await
    {
        Ok(assets) => (assets, None),
        Err(error) => (AssetManager::default(), Some(error)),
    };

    let mut game_data = GameData::new(assets, startup_warning);

    let mut state_machine: StateMachine<GameData, AppState> = StateMachine::new();
    state_machine.add_state(AppState::Playing, PlayingState);
    state_machine.add_state(AppState::Paused, PausedState);

    if let Err(error) = state_machine.set_initial(AppState::Playing, &mut game_data) {
        eprintln!("{error}");
        return;
    }

    loop {
        if is_key_pressed(KeyCode::F3) {
            toggle_debug();
        }

        clear_background(Color::from_rgba(20, 23, 33, 255));

        state_machine.update(&mut game_data);
        state_machine.draw(&game_data);

        if debug_enabled() {
            let state_name = state_machine
                .current()
                .map(AppState::label)
                .unwrap_or("none");
            draw_debug_overlay(&game_data.world, state_name);
        }

        next_frame().await;
    }
}
