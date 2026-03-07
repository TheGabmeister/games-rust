#[path = "entities.rs"]
pub mod entities;
#[path = "render.rs"]
mod render;
#[path = "state.rs"]
mod state;
#[path = "systems.rs"]
mod systems;

pub use state::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
