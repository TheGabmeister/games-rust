use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use hecs::{Entity, World};
use macroquad::audio::{Sound, load_sound, play_sound_once};
use macroquad::file::load_file;
use macroquad::prelude::*;
use serde::Deserialize;

const ASSET_MANIFEST_PATH: &str = "assets/assets.json";
const PLAYING_STATE: &str = "playing";
const PAUSED_STATE: &str = "paused";

static DEBUG: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
struct AssetManifest {
    #[serde(default)]
    textures: Vec<AssetEntry>,
    #[serde(default)]
    audio: Vec<AssetEntry>,
}

#[derive(Debug, Deserialize)]
struct AssetEntry {
    name: String,
    path: String,
}

#[derive(Default)]
struct AssetManager {
    textures: HashMap<String, Texture2D>,
    sounds: HashMap<String, Sound>,
    warnings: Vec<String>,
}

impl AssetManager {
    async fn from_manifest(path: &str) -> Result<Self, String> {
        let bytes = load_file(path)
            .await
            .map_err(|e| format!("Failed to read asset manifest at '{path}': {e}"))?;
        let text = String::from_utf8(bytes)
            .map_err(|e| format!("Asset manifest is not valid UTF-8: {e}"))?;
        let manifest: AssetManifest =
            serde_json::from_str(&text).map_err(|e| format!("Invalid asset JSON: {e}"))?;

        let mut assets = Self::default();

        for entry in manifest.textures {
            match load_texture(&entry.path).await {
                Ok(texture) => {
                    texture.set_filter(FilterMode::Nearest);
                    assets.textures.insert(entry.name, texture);
                }
                Err(e) => {
                    assets.warnings.push(format!(
                        "Texture '{}' failed to load from '{}': {e}",
                        entry.name, entry.path
                    ));
                }
            }
        }

        for entry in manifest.audio {
            match load_sound(&entry.path).await {
                Ok(sound) => {
                    assets.sounds.insert(entry.name, sound);
                }
                Err(e) => {
                    assets.warnings.push(format!(
                        "Audio '{}' failed to load from '{}': {e}",
                        entry.name, entry.path
                    ));
                }
            }
        }

        Ok(assets)
    }

    fn texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    fn sound(&self, name: &str) -> Option<&Sound> {
        self.sounds.get(name)
    }

    fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

trait GameState<C> {
    fn on_enter(&mut self, _ctx: &mut C) {}
    fn on_exit(&mut self, _ctx: &mut C) {}
    fn update(&mut self, ctx: &mut C) -> Option<String>;
    fn draw(&self, ctx: &C);
}

struct StateMachine<C> {
    states: HashMap<String, Box<dyn GameState<C>>>,
    current: Option<String>,
}

impl<C> StateMachine<C> {
    fn new() -> Self {
        Self {
            states: HashMap::new(),
            current: None,
        }
    }

    fn add_state<S: GameState<C> + 'static>(&mut self, name: &str, state: S) {
        self.states.insert(name.to_owned(), Box::new(state));
    }

    fn set_initial(&mut self, name: &str, ctx: &mut C) -> Result<(), String> {
        if !self.states.contains_key(name) {
            return Err(format!("State '{name}' is not registered"));
        }

        self.current = Some(name.to_owned());
        if let Some(state) = self.states.get_mut(name) {
            state.on_enter(ctx);
        }

        Ok(())
    }

    fn current_name(&self) -> &str {
        self.current.as_deref().unwrap_or("none")
    }

    fn transition(&mut self, next: &str, ctx: &mut C) {
        if self.current.as_deref() == Some(next) || !self.states.contains_key(next) {
            return;
        }

        if let Some(current) = self.current.clone() {
            if let Some(state) = self.states.get_mut(&current) {
                state.on_exit(ctx);
            }
        }

        if let Some(state) = self.states.get_mut(next) {
            state.on_enter(ctx);
        }

        self.current = Some(next.to_owned());
    }

    fn update(&mut self, ctx: &mut C) {
        let Some(current) = self.current.clone() else {
            return;
        };

        let next = if let Some(state) = self.states.get_mut(&current) {
            state.update(ctx)
        } else {
            None
        };

        if let Some(next_state) = next {
            self.transition(&next_state, ctx);
        }
    }

    fn draw(&self, ctx: &C) {
        if let Some(current) = &self.current {
            if let Some(state) = self.states.get(current) {
                state.draw(ctx);
            }
        }
    }
}

#[derive(Clone)]
struct Name(String);

#[derive(Clone, Copy)]
struct Transform {
    position: Vec2,
}

#[derive(Clone, Copy)]
struct Velocity {
    value: Vec2,
}

#[derive(Clone, Copy)]
struct Collider {
    size: Vec2,
}

#[derive(Clone)]
struct Sprite {
    size: Vec2,
    color: Color,
    texture: Option<String>,
}

#[derive(Clone, Copy)]
struct Bouncer;

#[derive(Clone, Copy)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self {
            min: position,
            max: position + size,
        }
    }
}

fn intersects(a: Aabb, b: Aabb) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

struct GameData {
    world: World,
    assets: AssetManager,
    player: Entity,
    last_collision_notes: Vec<String>,
    was_colliding: bool,
    startup_warning: Option<String>,
}

impl GameData {
    fn new(assets: AssetManager, startup_warning: Option<String>) -> Self {
        let mut world = World::new();

        let player = world.spawn((
            Name("Player".to_owned()),
            Transform {
                position: vec2(90.0, 120.0),
            },
            Velocity { value: Vec2::ZERO },
            Collider {
                size: vec2(48.0, 48.0),
            },
            Sprite {
                size: vec2(48.0, 48.0),
                color: SKYBLUE,
                texture: Some("player".to_owned()),
            },
        ));

        world.spawn((
            Name("MovingEnemy".to_owned()),
            Transform {
                position: vec2(420.0, 200.0),
            },
            Velocity {
                value: vec2(130.0, 100.0),
            },
            Collider {
                size: vec2(56.0, 56.0),
            },
            Sprite {
                size: vec2(56.0, 56.0),
                color: ORANGE,
                texture: Some("enemy".to_owned()),
            },
            Bouncer,
        ));

        world.spawn((
            Name("Wall".to_owned()),
            Transform {
                position: vec2(250.0, 320.0),
            },
            Collider {
                size: vec2(320.0, 34.0),
            },
            Sprite {
                size: vec2(320.0, 34.0),
                color: DARKGRAY,
                texture: None,
            },
        ));

        Self {
            world,
            assets,
            player,
            last_collision_notes: Vec::new(),
            was_colliding: false,
            startup_warning,
        }
    }

    fn update_playing(&mut self) {
        self.apply_player_input();
        self.run_movement_system();
        self.run_collision_system();

        if is_key_pressed(KeyCode::Space) {
            if let Some(sound) = self.assets.sound("blip") {
                play_sound_once(sound);
            }
        }
    }

    fn apply_player_input(&mut self) {
        let mut direction = Vec2::ZERO;

        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            direction.x += 1.0;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            direction.y += 1.0;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        if let Ok(mut velocity) = self.world.get::<&mut Velocity>(self.player) {
            velocity.value = direction * 260.0;
        }
    }

    fn run_movement_system(&mut self) {
        let dt = get_frame_time();

        for (transform, velocity) in self.world.query_mut::<(&mut Transform, &Velocity)>() {
            transform.position += velocity.value * dt;
        }

        for (transform, velocity, collider, _) in
            self.world
                .query_mut::<(&mut Transform, &mut Velocity, &Collider, &Bouncer)>()
        {
            if transform.position.x <= 0.0 {
                transform.position.x = 0.0;
                velocity.value.x = velocity.value.x.abs();
            } else if transform.position.x + collider.size.x >= screen_width() {
                transform.position.x = (screen_width() - collider.size.x).max(0.0);
                velocity.value.x = -velocity.value.x.abs();
            }

            if transform.position.y <= 0.0 {
                transform.position.y = 0.0;
                velocity.value.y = velocity.value.y.abs();
            } else if transform.position.y + collider.size.y >= screen_height() {
                transform.position.y = (screen_height() - collider.size.y).max(0.0);
                velocity.value.y = -velocity.value.y.abs();
            }
        }

        let mut player_query = self
            .world
            .query_one::<(&mut Transform, &Collider)>(self.player);
        if let Ok((transform, collider)) = player_query.get() {
            transform.position.x = transform
                .position
                .x
                .clamp(0.0, (screen_width() - collider.size.x).max(0.0));
            transform.position.y = transform
                .position
                .y
                .clamp(0.0, (screen_height() - collider.size.y).max(0.0));
        }
    }

    fn run_collision_system(&mut self) {
        self.last_collision_notes.clear();

        let Some(player_aabb) = self.entity_aabb(self.player) else {
            return;
        };

        for (entity, name, transform, collider) in self
            .world
            .query_mut::<(Entity, &Name, &Transform, &Collider)>()
        {
            if entity == self.player {
                continue;
            }

            let other = Aabb::from_position_size(transform.position, collider.size);
            if intersects(player_aabb, other) {
                self.last_collision_notes
                    .push(format!("Player collides with {}", name.0));
            }
        }

        let colliding = !self.last_collision_notes.is_empty();
        if colliding && !self.was_colliding {
            if let Some(sound) = self.assets.sound("hit") {
                play_sound_once(sound);
            }
        }
        self.was_colliding = colliding;
    }

    fn entity_aabb(&self, entity: Entity) -> Option<Aabb> {
        let transform = self.world.get::<&Transform>(entity).ok()?;
        let collider = self.world.get::<&Collider>(entity).ok()?;
        Some(Aabb::from_position_size(transform.position, collider.size))
    }

    fn draw_world(&self) {
        let mut query = self.world.query::<(&Transform, &Sprite)>();
        for (transform, sprite) in query.iter() {
            if let Some(texture_name) = &sprite.texture {
                if let Some(texture) = self.assets.texture(texture_name) {
                    draw_texture_ex(
                        texture,
                        transform.position.x,
                        transform.position.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(sprite.size),
                            ..Default::default()
                        },
                    );
                    continue;
                }
            }

            draw_rectangle(
                transform.position.x,
                transform.position.y,
                sprite.size.x,
                sprite.size.y,
                sprite.color,
            );
        }
    }

    fn draw_ui(&self) {
        draw_text(
            "Move: WASD/Arrows | Pause: P/Esc | Toggle Debug: F3 | Blip: Space",
            16.0,
            28.0,
            24.0,
            WHITE,
        );

        let mut text_y = 56.0;

        for note in &self.last_collision_notes {
            draw_text(note, 16.0, text_y, 24.0, YELLOW);
            text_y += 24.0;
        }

        if let Some(warning) = &self.startup_warning {
            draw_text(warning, 16.0, screen_height() - 22.0, 20.0, RED);
        }

        let mut warning_y = screen_height() - 48.0;
        for warning in self.assets.warnings().iter().take(2) {
            draw_text(warning, 16.0, warning_y, 20.0, ORANGE);
            warning_y -= 24.0;
        }
    }
}

struct PlayingState;

impl GameState<GameData> for PlayingState {
    fn update(&mut self, ctx: &mut GameData) -> Option<String> {
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            return Some(PAUSED_STATE.to_owned());
        }

        ctx.update_playing();
        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();
    }
}

struct PausedState;

impl GameState<GameData> for PausedState {
    fn update(&mut self, _ctx: &mut GameData) -> Option<String> {
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            return Some(PLAYING_STATE.to_owned());
        }

        None
    }

    fn draw(&self, ctx: &GameData) {
        ctx.draw_world();
        ctx.draw_ui();

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );

        let title = "PAUSED";
        let title_size = 64.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            (screen_width() - title_width) * 0.5,
            screen_height() * 0.5 - 10.0,
            title_size,
            WHITE,
        );
        draw_text(
            "Press P or Esc to resume",
            screen_width() * 0.5 - 140.0,
            screen_height() * 0.5 + 28.0,
            28.0,
            LIGHTGRAY,
        );
    }
}

fn draw_debug_overlay(world: &World, state_name: &str) {
    let mut entity_lines = Vec::new();
    let mut query = world.query::<(Entity, &Name)>();
    for (entity, name) in query.iter() {
        entity_lines.push(format!("{entity:?} | {}", name.0));
    }
    entity_lines.sort();

    let mut lines = Vec::new();
    lines.push("DEBUG MENU".to_owned());
    lines.push(format!("State: {state_name}"));
    lines.push(format!("Entity count: {}", world.len()));
    lines.extend(entity_lines);

    let panel_width = 360.0;
    let line_height = 20.0;
    let panel_height = 18.0 + line_height * lines.len() as f32 + 10.0;
    let float_offset = (get_time() as f32 * 2.2).sin() * 4.0;
    let panel_x = screen_width() - panel_width - 16.0;
    let panel_y = 18.0 + float_offset;

    draw_rectangle(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.06, 0.08, 0.1, 0.86),
    );
    draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 1.0, GREEN);

    for (i, line) in lines.iter().enumerate() {
        draw_text(
            line,
            panel_x + 10.0,
            panel_y + 24.0 + i as f32 * line_height,
            20.0,
            WHITE,
        );
    }
}

#[macroquad::main("Macroquad + hecs Template")]
async fn main() {
    let (assets, startup_warning) = match AssetManager::from_manifest(ASSET_MANIFEST_PATH).await {
        Ok(assets) => (assets, None),
        Err(e) => (AssetManager::default(), Some(e)),
    };

    let mut game_data = GameData::new(assets, startup_warning);

    let mut state_machine = StateMachine::new();
    state_machine.add_state(PLAYING_STATE, PlayingState);
    state_machine.add_state(PAUSED_STATE, PausedState);

    if let Err(e) = state_machine.set_initial(PLAYING_STATE, &mut game_data) {
        eprintln!("{e}");
        return;
    }

    loop {
        if is_key_pressed(KeyCode::F3) {
            DEBUG.store(!DEBUG.load(Ordering::Relaxed), Ordering::Relaxed);
        }

        clear_background(Color::from_rgba(20, 23, 33, 255));

        state_machine.update(&mut game_data);
        state_machine.draw(&game_data);

        if DEBUG.load(Ordering::Relaxed) {
            draw_debug_overlay(&game_data.world, state_machine.current_name());
        }

        next_frame().await;
    }
}
