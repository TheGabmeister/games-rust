use macroquad::audio::{PlaySoundParams, Sound, play_sound};
use std::collections::HashMap;

use crate::events::SfxId;

use super::Assets;

pub struct SfxManager {
    sounds: HashMap<SfxId, Sound>,
}

impl SfxManager {
    pub fn new(assets: &Assets) -> Self {
        Self {
            sounds: assets.sfx_bank().clone(),
        }
    }

    pub fn play_sound(&self, id: SfxId) {
        if let Some(sound) = self.sounds.get(&id) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: 1.0,
                },
            );
        }
    }
}
