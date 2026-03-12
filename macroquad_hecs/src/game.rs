use hecs::World;

use crate::events::GameEvent;
use crate::managers::Assets;
use crate::prefabs;
use crate::resources::{GameState, Resources};
use crate::systems::{self, render};

pub struct Game {
    world: World,
    res: Resources,
}

fn spawn_initial_wave(world: &mut World) {
    prefabs::spawn_player(world);

    // Spawn a few enemies to demonstrate the template.
    prefabs::spawn_enemy(
        world,
        crate::components::EnemyKind::Black,
        macroquad::prelude::vec2(150.0, 100.0),
    );
    prefabs::spawn_enemy(
        world,
        crate::components::EnemyKind::Blue,
        macroquad::prelude::vec2(300.0, 150.0),
    );
    prefabs::spawn_enemy(
        world,
        crate::components::EnemyKind::Green,
        macroquad::prelude::vec2(450.0, 100.0),
    );
}

impl Game {
    /// Load all assets and set up the initial world state.
    /// Must be called from an async context (macroquad main is already async).
    pub async fn new() -> Self {
        let assets = Assets::load().await;
        let mut res = Resources::new(assets);
        let mut world = World::new();

        spawn_initial_wave(&mut world);
        res.events.emit(GameEvent::GameStarted);

        Self { world, res }
    }

    /// Fixed-timestep update (called at 60 Hz).
    pub fn update(&mut self, dt: f32) {
        systems::system_capture_input(&mut self.res.input);

        // Debug toggle
        if self.res.input.debug_toggle_pressed {
            self.res.director.debug_mode = !self.res.director.debug_mode;
        }

        if self.res.director.state == GameState::Playing {
            systems::system_player_movement(&mut self.world, &self.res.input, dt);
            systems::system_player_fire(&mut self.world, &self.res.input, &self.res.sfx, dt);

            systems::system_enemy_movement(&mut self.world);
            systems::system_enemy_fire(&mut self.world, &self.res.sfx, dt);

            systems::system_integrate(&mut self.world, dt);
            systems::system_cull_offscreen(&self.world, &mut self.res.despawns);
            systems::system_lifetime(&mut self.world, &mut self.res.despawns, dt);
            systems::system_apply_despawns(&mut self.world, &mut self.res.despawns);

            systems::system_collision(&self.world, &mut self.res.events, &mut self.res.despawns);

            // React to events (score, despawns, state transitions)
            systems::system_process_events(
                &mut self.world,
                &mut self.res.director,
                &mut self.res.events,
                &mut self.res.despawns,
                &mut self.res.sfx,
                &mut self.res.music,
            );
            systems::system_apply_despawns(&mut self.world, &mut self.res.despawns);
        } else if self.res.input.confirm_pressed {
            self.restart_run();
        }
    }

    /// Render (called every frame — not fixed-timestep).
    pub fn draw(&self) {
        render::draw(&self.world, &self.res.assets);

        #[cfg(debug_assertions)]
        if self.res.director.debug_mode {
            systems::system_draw_colliders(&self.world);
        }

        render::draw_hud(&self.res.director);
    }

    fn restart_run(&mut self) {
        self.world.clear();
        spawn_initial_wave(&mut self.world);
        self.res.events.drain();
        self.res.despawns.clear();
        self.res.director.reset_run();
    }
}
