use macroquad::audio::{play_sound, set_sound_volume, stop_sound, PlaySoundParams};

use crate::events::{MusicCommand, MusicId};
use crate::resources::Resources;

pub struct SfxManager {

}


impl SfxManager {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MusicManager {
    pub current: Option<MusicId>,
}


impl MusicManager {
    pub fn new() -> Self {
        Self { current: None }
    }
}
