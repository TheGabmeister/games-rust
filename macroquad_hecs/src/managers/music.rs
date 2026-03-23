use macroquad::audio::{PlaySoundParams, Sound, play_sound, stop_sound};
use std::collections::HashMap;

use crate::events::MusicId;

use super::Assets;

pub struct MusicManager {
    tracks: HashMap<MusicId, Sound>,
    current: Option<MusicId>,
}

impl MusicManager {
    pub fn new(assets: &Assets) -> Self {
        Self {
            tracks: assets.music_bank().clone(),
            current: None,
        }
    }

    pub fn play_music(&mut self, id: MusicId) {
        // Stop the currently playing track first.
        if let Some(cur) = self.current {
            if let Some(sound) = self.tracks.get(&cur) {
                stop_sound(sound);
            }
        }

        if let Some(sound) = self.tracks.get(&id) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 1.0,
                },
            );
            self.current = Some(id);
        }
    }
}
