use hecs::World;

use crate::events::{
    EnemyDestroyed, GameStarted, PickupCollected, PlayMusic, PlaySfx, PlayerDied,
    PowerupCollected, StageCleared,
};
use crate::handlers;
use crate::managers::{AnimationDb, Assets};
use crate::prefabs;
use crate::resources::{GameState, Resources};
use crate::systems::{self, render};

pub struct Game {
    world: World,
    res: Resources,
}

fn spawn_entities(world: &mut World, anim_db: &AnimationDb) {
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

    // Initial collectibles/powerups at run start.
    prefabs::spawn_pickup(
        world,
        crate::components::PickupKind::Life,
        macroquad::prelude::vec2(180.0, 220.0),
    );
    prefabs::spawn_pickup(
        world,
        crate::components::PickupKind::Star,
        macroquad::prelude::vec2(420.0, 220.0),
    );
    prefabs::spawn_powerup(
        world,
        crate::components::PowerupEffect::Bolt,
        macroquad::prelude::vec2(260.0, 280.0),
    );
    prefabs::spawn_powerup(
        world,
        crate::components::PowerupEffect::Shield,
        macroquad::prelude::vec2(340.0, 280.0),
    );

    // Animated character demo — cycles through all clips.
    prefabs::spawn_old_hero(world, anim_db, macroquad::prelude::vec2(300.0, 400.0));
}

impl Game {
    /// Load all assets and set up the initial world state.
    /// Must be called from an async context (macroquad main is already async).
    pub async fn new() -> Self {
        let assets = Assets::load().await;
        let mut res = Resources::new(assets);
        let mut world = World::new();

        // Register event handlers (observer pattern).
        res.event_registry.on::<GameStarted>(handlers::on_game_started);
        res.event_registry.on::<EnemyDestroyed>(handlers::on_enemy_destroyed);
        res.event_registry.on::<PlayerDied>(handlers::on_player_died);
        res.event_registry.on::<PickupCollected>(handlers::on_pickup_collected);
        res.event_registry.on::<PowerupCollected>(handlers::on_powerup_collected);
        res.event_registry.on::<StageCleared>(handlers::on_stage_cleared);
        res.event_registry.on::<PlaySfx>(handlers::on_play_sfx);
        res.event_registry.on::<PlayMusic>(handlers::on_play_music);

        spawn_entities(&mut world, &res.anim_db);

        res.events.emit(GameStarted);

        Self { world, res }
    }

    pub fn capture_input(&mut self) {
        systems::system_capture_input(&mut self.res.input);
    }

    /// Fixed-timestep update (called at 60 Hz).
    pub fn update(&mut self, dt: f32) {
        // Debug toggle
        if self.res.input.debug_toggle_pressed {
            self.res.director.debug_mode = !self.res.director.debug_mode;
        }

        if self.res.director.state == GameState::Playing {
            systems::system_tick_powerups(&mut self.world, dt);
            systems::system_animate(&mut self.world, &self.res.anim_db, dt);
            systems::system_anim_demo(&mut self.world, &self.res.anim_db, dt);
            systems::system_player_movement(&mut self.world, &self.res.input, dt);
            systems::system_player_fire(
                &mut self.world,
                &self.res.input,
                &mut self.res.events,
                dt,
            );

            systems::system_enemy_movement(&mut self.world);
            systems::system_enemy_fire(&mut self.world, &mut self.res.events, dt);

            systems::system_integrate(&mut self.world, dt);
            systems::system_cull_offscreen(&self.world, &mut self.res.despawns);
            systems::system_lifetime(&mut self.world, &mut self.res.despawns, dt);
            systems::system_apply_despawns(&mut self.world, &mut self.res.despawns);

            systems::system_collision(&self.world, &mut self.res.events, &mut self.res.despawns);

            // React to events (score, despawns, state transitions)
            systems::system_process_events(
                &mut self.world,
                &mut self.res.events,
                &self.res.event_registry,
                &mut self.res.director,
                &mut self.res.despawns,
                &mut self.res.sfx,
                &mut self.res.music,
            );
            systems::system_apply_despawns(&mut self.world, &mut self.res.despawns);
        } else if self.res.input.confirm_pressed {
            self.restart_run();
        }

        self.res.input.clear_transients();
    }

    /// Render (called every frame — not fixed-timestep).
    pub fn draw(&self) {
        render::draw(&self.world, &self.res.assets);
        render::draw_hud(&self.res.director);

        #[cfg(debug_assertions)]
        if self.res.director.debug_mode {
            systems::system_draw_colliders(&self.world);
            systems::system_draw_debug_ui(&self.world);
            egui_macroquad::draw();
        }
    }

    fn restart_run(&mut self) {
        self.world.clear();
        spawn_entities(&mut self.world, &self.res.anim_db);
        self.res.events.drain_raw();
        self.res.despawns.clear();
        self.res.director.reset_run();
    }
}
