use hecs::World;
use macroquad::audio::{play_sound, PlaySoundParams};

use crate::assets::load_all_assets;
use crate::events::{GameEvent, SoundId};
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

        prefabs::spawn_player(&mut world, &res);

        // Spawn a few enemies to demonstrate the template
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Black, macroquad::prelude::vec2(150.0, 100.0));
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Blue,  macroquad::prelude::vec2(300.0, 150.0));
        prefabs::spawn_enemy(&mut world, crate::components::EnemyKind::Green, macroquad::prelude::vec2(450.0, 100.0));

        // Start background music
        play_sound(
            res.sound(SoundId::MusicSpaceshooter),
            PlaySoundParams {
                looped: true,
                volume: 0.4,
            },
        );

        res.events.emit(GameEvent::GameStarted);

        Self { world, res }
    }

    /// Fixed-timestep update (called at 60 Hz).
    pub fn update(&mut self, dt: f32) {
        // 1. Capture input (must be first — all systems read res.input)
        systems::system_capture_input(&mut self.res.input);

        // 2. Player intent
        systems::system_player_movement(&mut self.world, &self.res, dt);
        systems::system_player_fire(&mut self.world, &mut self.res, dt);

        // 3. Enemy AI
        systems::system_enemy_movement(&mut self.world);
        systems::system_enemy_fire(&mut self.world, &mut self.res, dt);

        // 4. Physics
        systems::system_integrate(&mut self.world, dt);
        systems::system_cull_offscreen(&mut self.world);
        systems::system_lifetime(&mut self.world, dt);

        // 5. Collision → events
        systems::system_collision(&mut self.world, &mut self.res);

        // 6. React to events (health, score, despawns, re-emits)
        systems::system_process_events(&mut self.world, &mut self.res);

        // 7. Play queued sounds
        systems::system_audio(&mut self.res);

        // 8. Debug toggle
        if self.res.input.debug_toggle_pressed {
            self.res.debug_mode = !self.res.debug_mode;
        }
    }

    /// Render (called every frame — not fixed-timestep).
    pub fn draw(&self) {
        render::draw(&self.world, &self.res);

        #[cfg(debug_assertions)]
        if self.res.debug_mode {
            systems::system_draw_colliders(&self.world);
        }

        render::draw_hud(&self.res);
    }
}
