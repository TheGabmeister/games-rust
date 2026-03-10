use std::sync::{Mutex, OnceLock};

use macroquad::audio::{play_sound, set_sound_volume, stop_sound, PlaySoundParams};

use crate::events::MusicCommand;
use crate::resources::Resources;

pub struct SfxManager {
    volume: f32,
}

impl Default for SfxManager {
    fn default() -> Self {
        Self { volume: 0.8 }
    }
}

impl SfxManager {
    pub fn instance() -> &'static Mutex<Self> {
        static INSTANCE: OnceLock<Mutex<SfxManager>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(SfxManager::default()))
    }

    pub fn update(res: &mut Resources) {
        let mut manager = Self::instance()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        manager.process(res);
    }

    fn process(&mut self, res: &mut Resources) {
        let queue = std::mem::take(&mut res.sfx_queue);

        for id in queue {
            let sound = res.sfx(id);
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume,
                },
            );
        }
    }
}

pub struct MusicManager {
    current_music: Option<crate::events::MusicId>,
    volume: f32,
}

impl Default for MusicManager {
    fn default() -> Self {
        Self {
            current_music: None,
            volume: 0.4,
        }
    }
}

impl MusicManager {
    pub fn instance() -> &'static Mutex<Self> {
        static INSTANCE: OnceLock<Mutex<MusicManager>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(MusicManager::default()))
    }

    pub fn update(res: &mut Resources) {
        let mut manager = Self::instance()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        manager.process(res);
    }

    fn process(&mut self, res: &mut Resources) {
        let commands = std::mem::take(&mut res.music_queue);

        for command in commands {
            match command {
                MusicCommand::Play(id) => {
                    if self.current_music == Some(id) {
                        let sound = res.music(id);
                        set_sound_volume(sound, self.volume);
                        continue;
                    }

                    if let Some(current_id) = self.current_music {
                        let current = res.music(current_id);
                        stop_sound(current);
                    }

                    let next = res.music(id);
                    play_sound(
                        next,
                        PlaySoundParams {
                            looped: true,
                            volume: self.volume,
                        },
                    );
                    self.current_music = Some(id);
                }

                MusicCommand::Stop => {
                    if let Some(current_id) = self.current_music {
                        let current = res.music(current_id);
                        stop_sound(current);
                        self.current_music = None;
                    }
                }

                MusicCommand::SetVolume(volume) => {
                    self.volume = volume.clamp(0.0, 1.0);

                    if let Some(current_id) = self.current_music {
                        let current = res.music(current_id);
                        set_sound_volume(current, self.volume);
                    }
                }
            }
        }
    }
}
