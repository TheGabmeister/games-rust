use std::collections::HashMap;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use crate::events::SfxId;

pub struct SfxManager {
    sounds: HashMap<SfxId, Sound>,
}

impl SfxManager {
    pub fn new(sounds: HashMap<SfxId, Sound>) -> Self {
        Self { sounds }
    }

    pub fn play_sound(&self, id: SfxId) {
        if let Some(s) = self.sounds.get(&id) {
            play_sound(s, PlaySoundParams { looped: false, volume: 1.0 });
        }
    }
}