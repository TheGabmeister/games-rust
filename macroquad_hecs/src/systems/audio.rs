use macroquad::audio::{play_sound, set_sound_volume, stop_sound, PlaySoundParams};

use crate::events::MusicCommand;
use crate::resources::Resources;

/// Drain the SFX queue and play each queued one-shot sound.
///
/// Uses `std::mem::take` to move the Vec out of `res` before borrowing sound assets,
/// avoiding a simultaneous mutable + immutable borrow of `Resources`.
pub fn system_sfx(res: &mut Resources) {
    let queue = std::mem::take(&mut res.sfx_queue);

    for id in queue {
        let sound = res.sfx(id);
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );
    }
}

/// Drain music commands and apply them to the current track state.
pub fn system_music(res: &mut Resources) {
    let commands = std::mem::take(&mut res.music_queue);

    for command in commands {
        match command {
            MusicCommand::Play(id) => {
                if res.current_music == Some(id) {
                    let sound = res.music(id);
                    set_sound_volume(sound, res.music_volume);
                    continue;
                }

                if let Some(current_id) = res.current_music {
                    let current = res.music(current_id);
                    stop_sound(current);
                }

                let next = res.music(id);
                play_sound(
                    next,
                    PlaySoundParams {
                        looped: true,
                        volume: res.music_volume,
                    },
                );
                res.current_music = Some(id);
            }

            MusicCommand::Stop => {
                if let Some(current_id) = res.current_music {
                    let current = res.music(current_id);
                    stop_sound(current);
                    res.current_music = None;
                }
            }

            MusicCommand::SetVolume(volume) => {
                res.music_volume = volume.clamp(0.0, 1.0);

                if let Some(current_id) = res.current_music {
                    let current = res.music(current_id);
                    set_sound_volume(current, res.music_volume);
                }
            }
        }
    }
}
