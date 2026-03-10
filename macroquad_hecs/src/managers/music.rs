use macroquad::audio::{PlaySoundParams, Sound, play_sound};
use std::collections::HashMap;

use crate::events::MusicId;

use super::Assets;

pub struct MusicManager {
    tracks: HashMap<MusicId, Sound>,
}

impl MusicManager {
    pub fn new(assets: &Assets) -> Self {
        Self {
            tracks: assets.music_bank().clone(),
        }
    }

    pub fn play_music(&mut self, id: MusicId) {
        if let Some(sound) = self.tracks.get(&id) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 1.0,
                },
            );
        }
    }
}
