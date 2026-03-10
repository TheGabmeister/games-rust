use std::collections::HashMap;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use crate::events::MusicId;

pub struct MusicManager {

}

impl MusicManager {

    pub fn play_music(&mut self, &Assets) {
            play_sound(s, PlaySoundParams { looped: true, volume: 1.0 });
        }
    }
}
