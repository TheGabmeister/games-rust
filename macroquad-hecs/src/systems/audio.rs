use hecs::World;
use macroquad::audio::play_sound_once;

use crate::assets::{AssetManager, SoundId};
use crate::ecs::{CollisionState, Player};

pub fn play_blip(assets: &AssetManager) {
    if let Some(sound) = assets.sound(SoundId::Blip) {
        play_sound_once(sound);
    }
}

pub fn play_hit(assets: &AssetManager) {
    if let Some(sound) = assets.sound(SoundId::Hit) {
        play_sound_once(sound);
    }
}

/// Plays the hit sound if the player's `CollisionState` indicates a new
/// collision this frame. Reads the ECS directly — no external state needed.
pub fn play_hit_if_collision_started(world: &World, assets: &AssetManager) {
    let mut q = world.query::<(&Player, &CollisionState)>();
    if let Some((_, state)) = q.iter().next() {
        if state.started_colliding {
            play_hit(assets);
        }
    }
}
