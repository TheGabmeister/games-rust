use hecs::{Entity, World};
use macroquad::audio::play_sound_once;
use macroquad::prelude::*;

use crate::assets::AssetManager;
use crate::collision::{Aabb, intersects};
use crate::ecs::{
    Bouncer, Collider, Name, PLAYER_SPEED, Sprite, Transform, Velocity, spawn_template_entities,
};

pub struct GameData {
    pub world: World,
    pub assets: AssetManager,
    player: Entity,
    last_collision_notes: Vec<String>,
    was_colliding: bool,
    startup_warning: Option<String>,
}

impl GameData {
    pub fn new(assets: AssetManager, startup_warning: Option<String>) -> Self {
        let mut world = World::new();
        let player = spawn_template_entities(&mut world);

        Self {
            world,
            assets,
            player,
            last_collision_notes: Vec::new(),
            was_colliding: false,
            startup_warning,
        }
    }

    pub fn update_playing(&mut self) {
        self.apply_player_input();
        self.run_movement_system();
        self.run_collision_system();

        if is_key_pressed(KeyCode::Space) {
            if let Some(sound) = self.assets.sound("blip") {
                play_sound_once(sound);
            }
        }
    }

    pub fn draw_world(&self) {
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

    pub fn draw_ui(&self) {
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
            velocity.value = direction * PLAYER_SPEED;
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
}
