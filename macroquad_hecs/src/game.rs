use hecs::World;

use crate::assets::load_all_assets;
use crate::events::GameEvent;
use crate::prefabs;
use crate::resources::Resources;
use crate::systems::{self, render};

pub struct Game {
    world: World,
    res: Resources,
}

impl Game {
    /// Load all assets and set up the initial world state.
    /// Must be called from an async context (macroquad main is already async).
    pub async fn new() -> Self {
        let assets = load_all_assets().await;
        let mut res = Resources::new(assets);
        let mut world = World::new();

        prefabs::spawn_player(&mut world);

        // Spawn a few enemies to demonstrate the template
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Black, macroquad::prelude::vec2(150.0, 100.0));
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Blue,  macroquad::prelude::vec2(300.0, 150.0));
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Green, macroquad::prelude::vec2(450.0, 100.0));

        // GameStarted event triggers music in system_process_events (first update tick)
        res.runtime.events.emit(GameEvent::GameStarted);

        Self {
            world,
            res,
        }
    }

    /// Fixed-timestep update (called at 60 Hz).
    pub fn update(&mut self, dt: f32) {
        // 1. Capture input (must be first — systems read runtime.input)
        systems::system_capture_input(&mut self.res.runtime.input);

        // 2. Player intent
        systems::system_player_movement(&mut self.world, &self.res.runtime.input, dt);
        systems::system_player_fire(
            &mut self.world,
            &self.res.runtime.input,
            &self.res.audio.sfx,
            dt,
        );

        // 3. Enemy AI
        systems::system_enemy_movement(&mut self.world);
        systems::system_enemy_fire(&mut self.world, &self.res.audio.sfx, dt);

        // 4. Physics
        systems::system_integrate(&mut self.world, dt);
        systems::system_cull_offscreen(&mut self.world);
        systems::system_lifetime(&mut self.world, dt);

        // 5. Collision → events
        systems::system_collision(&mut self.world, &mut self.res.runtime.events);

        // 6. React to events (score, despawns, re-emits)
        systems::system_process_events(
            &mut self.world,
            &mut self.res.state,
            &mut self.res.runtime.events,
            &mut self.res.audio,
        );

        // 7. Debug toggle
        if self.res.runtime.input.debug_toggle_pressed {
            self.res.state.debug_mode = !self.res.state.debug_mode;
        }
    }

    /// Render (called every frame — not fixed-timestep).
    pub fn draw(&self) {
        render::draw(&self.world, &self.res.textures);

        #[cfg(debug_assertions)]
        if self.res.state.debug_mode {
            systems::system_draw_colliders(&self.world);
        }

        render::draw_hud(&self.res.state);
    }
}
