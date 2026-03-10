use std::collections::HashMap;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use crate::events::MusicId;

pub struct MusicManager {
    pub current: Option<MusicId>,
    sounds: HashMap<MusicId, Sound>,
}

impl MusicManager {
    pub fn new(sounds: HashMap<MusicId, Sound>) -> Self {
        Self { current: None, sounds }
    }

    pub fn play_music(&mut self, id: MusicId) {
        if let Some(s) = self.sounds.get(&id) {
            play_sound(s, PlaySoundParams { looped: true, volume: 1.0 });
        }
        self.current = Some(id);
    }
}
