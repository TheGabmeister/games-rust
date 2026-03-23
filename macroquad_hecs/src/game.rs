use hecs::World;
use macroquad::prelude::*;

use crate::campaign::Campaign;
use crate::events::{
    EnemyDestroyed, GameStarted, PickupCollected, PlayMusic, PlaySfx, PlayerDied,
    PowerupCollected, StageCleared,
};
use crate::handlers;
use crate::managers::Assets;
use crate::resources::{GameState, Resources};
use crate::scene::{enter_character_scene, enter_gameplay_scene, load_scene_def, ActiveScene, SceneDef};
use crate::systems::{self, render};

pub struct Game {
    world: World,
    res: Resources,
    campaign: Campaign,
    active_scene: ActiveScene,
    demo_scene: SceneDef,
    menu_selection: usize,
}

impl Game {
    pub async fn new() -> Self {
        let assets = Assets::load().await;
        let mut res = Resources::new(assets);
        let world = World::new();

        // Register event handlers (observer pattern).
        res.event_registry.on::<GameStarted>(handlers::on_game_started);
        res.event_registry.on::<EnemyDestroyed>(handlers::on_enemy_destroyed);
        res.event_registry.on::<PlayerDied>(handlers::on_player_died);
        res.event_registry.on::<PickupCollected>(handlers::on_pickup_collected);
        res.event_registry.on::<PowerupCollected>(handlers::on_powerup_collected);
        res.event_registry.on::<StageCleared>(handlers::on_stage_cleared);
        res.event_registry.on::<PlaySfx>(handlers::on_play_sfx);
        res.event_registry.on::<PlayMusic>(handlers::on_play_music);

        let campaign = Campaign::load("assets/scenes");
        let demo_scene = load_scene_def("assets/scenes/demo_character.ron");

        Self {
            world,
            res,
            campaign,
            active_scene: ActiveScene::Menu,
            demo_scene,
            menu_selection: 0,
        }
    }

    pub fn capture_input(&mut self) {
        systems::system_capture_input(&mut self.res.input);
    }

    /// Fixed-timestep update (called at 60 Hz).
    pub fn update(&mut self, dt: f32) {
        // Debug toggle (always available)
        if self.res.input.debug_toggle_pressed {
            self.res.director.debug_mode = !self.res.director.debug_mode;
        }

        match self.active_scene {
            ActiveScene::Menu => self.update_menu(),
            ActiveScene::Gameplay => self.update_gameplay(dt),
            ActiveScene::CharacterDemo => self.update_character_demo(dt),
        }

        self.res.input.clear_transients();
    }

    fn update_menu(&mut self) {
        if self.res.input.nav_up_pressed && self.menu_selection > 0 {
            self.menu_selection -= 1;
        }
        if self.res.input.nav_down_pressed && self.menu_selection < 1 {
            self.menu_selection += 1;
        }

        if self.res.input.confirm_pressed {
            match self.menu_selection {
                0 => {
                    self.res.director.reset_run();
                    self.campaign.current_index = 0;
                    enter_gameplay_scene(
                        &mut self.world,
                        &mut self.res,
                        self.campaign.current_scene(),
                    );
                    self.active_scene = ActiveScene::Gameplay;
                }
                1 => {
                    enter_character_scene(&mut self.world, &mut self.res, &self.demo_scene);
                    self.active_scene = ActiveScene::CharacterDemo;
                }
                _ => {}
            }
        }
    }

    fn update_character_demo(&mut self, dt: f32) {
        systems::system_animate(&mut self.world, &self.res.anim_db, dt);
        systems::system_anim_demo(&mut self.world, &self.res.anim_db, dt);

        if self.res.input.cancel_pressed {
            self.world.clear();
            self.active_scene = ActiveScene::Menu;
        }
    }

    fn update_gameplay(&mut self, dt: f32) {
        match self.res.director.state {
            GameState::Playing => {
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

                systems::system_collision(
                    &self.world,
                    &mut self.res.events,
                    &mut self.res.despawns,
                );

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

                // After event processing, check if stage was cleared and advance campaign.
                if self.res.director.state == GameState::Won {
                    if self.campaign.has_next() {
                        self.campaign.advance();
                        enter_gameplay_scene(
                            &mut self.world,
                            &mut self.res,
                            self.campaign.current_scene(),
                        );
                    }
                    // else: final scene cleared — stay in Won state, draw() shows overlay
                }
            }
            GameState::Won | GameState::Lost => {
                if self.res.input.confirm_pressed {
                    self.world.clear();
                    self.active_scene = ActiveScene::Menu;
                }
            }
        }
    }

    /// Render (called every frame — not fixed-timestep).
    pub fn draw(&self) {
        match self.active_scene {
            ActiveScene::Menu => self.draw_menu(),
            ActiveScene::Gameplay => self.draw_gameplay(),
            ActiveScene::CharacterDemo => self.draw_character_demo(),
        }
    }

    fn draw_menu(&self) {
        use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

        clear_background(Color::from_hex(0x0a0a1a));

        let title = "MACROQUAD HECS";
        let dim = measure_text(title, None, 48, 1.0);
        draw_text(
            title,
            (SCREEN_WIDTH - dim.width) * 0.5,
            SCREEN_HEIGHT * 0.30,
            48.0,
            WHITE,
        );

        let options = ["START GAME", "CHARACTER DEMO"];
        for (i, label) in options.iter().enumerate() {
            let y = SCREEN_HEIGHT * 0.48 + i as f32 * 50.0;
            let dim_opt = measure_text(label, None, 28, 1.0);
            let color = if i == self.menu_selection {
                WHITE
            } else {
                Color::from_hex(0x666666)
            };

            if i == self.menu_selection {
                draw_text(
                    ">",
                    (SCREEN_WIDTH - dim_opt.width) * 0.5 - 30.0,
                    y,
                    28.0,
                    WHITE,
                );
            }

            draw_text(
                label,
                (SCREEN_WIDTH - dim_opt.width) * 0.5,
                y,
                28.0,
                color,
            );
        }

        if self.res.director.high_score > 0 {
            let hs = format!("BEST: {}", self.res.director.high_score);
            let dim3 = measure_text(&hs, None, 22, 1.0);
            draw_text(
                &hs,
                (SCREEN_WIDTH - dim3.width) * 0.5,
                SCREEN_HEIGHT * 0.70,
                22.0,
                Color::from_hex(0xffd700),
            );
        }
    }

    fn draw_character_demo(&self) {
        use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

        render::draw(&self.world, &self.res.assets, self.demo_scene.background_color);

        let title = &self.demo_scene.name;
        let dim = measure_text(title, None, 36, 1.0);
        draw_text(
            title,
            (SCREEN_WIDTH - dim.width) * 0.5,
            SCREEN_HEIGHT * 0.08,
            36.0,
            WHITE,
        );

        let prompt = "PRESS ESC TO GO BACK";
        let dim2 = measure_text(prompt, None, 22, 1.0);
        draw_text(
            prompt,
            (SCREEN_WIDTH - dim2.width) * 0.5,
            SCREEN_HEIGHT * 0.95,
            22.0,
            Color::from_hex(0xaaaaaa),
        );
    }

    fn draw_gameplay(&self) {
        render::draw(&self.world, &self.res.assets, self.campaign.current_scene().background_color);
        render::draw_hud(&self.res.director);

        #[cfg(debug_assertions)]
        if self.res.director.debug_mode {
            systems::system_draw_colliders(&self.world);
            systems::system_draw_debug_ui(&self.world);
            egui_macroquad::draw();
        }
    }
}
