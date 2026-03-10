use std::collections::HashMap;

use macroquad::audio::{play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};

use crate::events::MusicId;

pub struct SfxManager {

}


impl SfxManager {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MusicManager {
    pub current: Option<MusicId>,
    sounds: HashMap<MusicId, Sound>,
}


impl MusicManager {
    pub fn new(sounds: HashMap<MusicId, Sound>) -> Self {
        Self { current: None, sounds }
    }

    pub fn play_music(&mut self, id: MusicId) {
        println!("play_music id: {:?}", id);
        play_sound(self.sounds.get(&id).unwrap(), PlaySoundParams { looped: true, volume: 1.0 });
  
    }

}
