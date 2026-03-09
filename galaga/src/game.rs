use hecs::World;

use crate::render;
use crate::resources::Resources;
use crate::systems;

pub struct Game {
    world: World,
    resources: Resources,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: Resources::default(),
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        systems::fixed_update(&mut self.world, &mut self.resources, dt);
    }

    pub fn draw(&self) {
        render::draw(&self.world, &self.resources);
    }
}
