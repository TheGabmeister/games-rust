use macroquad::audio::play_sound_once;

use crate::assets::{AssetManager, SoundId};

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
