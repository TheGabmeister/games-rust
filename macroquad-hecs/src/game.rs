use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::assets::{AssetManager, TextureId};
use crate::config::{
    FIXED_DT, MAX_FRAME_DT, MAX_SIM_STEPS_PER_FRAME, PLAYER_SPEED, WORLD_HEIGHT, WORLD_WIDTH,
};
use crate::ecs::{
    PreviousTransform, RenderLayer, RenderSpace, Sprite, Transform, spawn_template_entities,
};
use crate::render::CameraRig;
use crate::systems::input::InputState;
use crate::systems::movement::WorldBounds;
use crate::systems::{FixedTimestepScheduler, audio, collision, input, movement, ui};

#[derive(Clone, Copy)]
struct DrawItem {
    position: Vec2,
    size: Vec2,
    color: Color,
    texture: Option<TextureId>,
    layer: RenderLayer,
    space: RenderSpace,
}

pub struct GameData {
    pub world: World,
    pub assets: AssetManager,
    player: Entity,
    scheduler: FixedTimestepScheduler,
    world_bounds: WorldBounds,
    camera: CameraRig,
    last_collision_notes: Vec<String>,
    was_colliding: bool,
    startup_warning: Option<String>,
}

impl GameData {
    pub fn new(assets: AssetManager, startup_warning: Option<String>) -> Self {
        let mut world = World::new();
        let player = spawn_template_entities(&mut world);
        let world_bounds = WorldBounds::from_size(vec2(WORLD_WIDTH, WORLD_HEIGHT));

        Self {
            world,
            assets,
            player,
            scheduler: FixedTimestepScheduler::new(FIXED_DT, MAX_FRAME_DT, MAX_SIM_STEPS_PER_FRAME),
            world_bounds,
            camera: CameraRig::new(world_bounds.size()),
            last_collision_notes: Vec::new(),
            was_colliding: false,
            startup_warning,
        }
    }

    pub fn update_playing(&mut self, frame_dt: f32) {
        let input = input::sample_input();
        if input.blip_pressed {
            audio::play_blip(&self.assets);
        }

        let steps = self.scheduler.begin_frame(frame_dt);
        for _ in 0..steps {
            self.run_fixed_step(input);
        }
    }

    pub fn draw_world(&self) {
        let mut draw_items = self.collect_draw_items(self.scheduler.alpha());
        draw_items.sort_by_key(|item| item.layer.0);

        self.camera.begin_world_pass();
        self.draw_items_for_space(&draw_items, RenderSpace::World);

        self.camera.begin_screen_pass();
        self.draw_items_for_space(&draw_items, RenderSpace::Screen);
    }

    pub fn draw_ui(&self) {
        self.camera.begin_screen_pass();
        ui::draw_hud(
            &self.last_collision_notes,
            self.startup_warning.as_deref(),
            self.assets.warnings(),
        );
    }

    pub fn draw_paused_overlay(&self) {
        self.camera.begin_screen_pass();
        ui::draw_paused_overlay();
    }

    fn run_fixed_step(&mut self, input: InputState) {
        movement::snapshot_previous_transforms(&mut self.world);
        input::apply_player_velocity(&mut self.world, self.player, input, PLAYER_SPEED);
        movement::integrate(&mut self.world, self.scheduler.fixed_dt());
        movement::bounce(&mut self.world, self.world_bounds);
        movement::clamp_player(&mut self.world, self.player, self.world_bounds);

        let report =
            collision::detect_player_collisions(&self.world, self.player, self.was_colliding);
        self.last_collision_notes = report.notes;
        if report.started_colliding {
            audio::play_hit(&self.assets);
        }
        self.was_colliding = report.is_colliding;
    }

    fn collect_draw_items(&self, alpha: f32) -> Vec<DrawItem> {
        let mut items = Vec::new();
        let mut query = self
            .world
            .query::<(&Transform, Option<&PreviousTransform>, &Sprite)>();

        for (transform, previous, sprite) in query.iter() {
            let position = match previous {
                Some(previous) => previous.position.lerp(transform.position, alpha),
                None => transform.position,
            };

            items.push(DrawItem {
                position,
                size: sprite.size,
                color: sprite.color,
                texture: sprite.texture,
                layer: sprite.layer,
                space: sprite.space,
            });
        }

        items
    }

    fn draw_items_for_space(&self, items: &[DrawItem], space: RenderSpace) {
        for item in items {
            if item.space == space {
                self.draw_item(*item);
            }
        }
    }

    fn draw_item(&self, item: DrawItem) {
        if let Some(texture_id) = item.texture
            && let Some(texture) = self.assets.texture(texture_id)
        {
            draw_texture_ex(
                texture,
                item.position.x,
                item.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(item.size),
                    ..Default::default()
                },
            );
            return;
        }

        draw_rectangle(
            item.position.x,
            item.position.y,
            item.size.x,
            item.size.y,
            item.color,
        );
    }
}
