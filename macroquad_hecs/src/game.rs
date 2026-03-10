use hecs::World;

use crate::systems::render;
use crate::resources::Resources;

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

    pub fn update(&mut self, dt: f32) {

    }

    pub fn draw(&mut self) {
        render::draw(&self.world);
    }
}
