use std::collections::HashMap;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use crate::events::SfxId;

pub struct SfxManager {
    sounds: HashMap<SfxId, Sound>,
}

impl SfxManager {

    pub fn play_sound(&self, &Assets) {
            play_sound(s, PlaySoundParams { looped: false, volume: 1.0 });
    }
}