use macroquad::audio::{PlaySoundParams, play_sound, stop_sound};

use crate::resources::{Resources, SoundId};

pub fn start_music(res: &Resources) {
    play_sound(
        &res.music_spaceshooter,
        PlaySoundParams {
            looped: true,
            volume: 0.4,
        },
    );
}

pub fn stop_music(res: &Resources) {
    stop_sound(&res.music_spaceshooter);
}

/// Drain the audio queue and play each pending sound.
/// Collecting first avoids a simultaneous mutable borrow of audio_queue
/// and immutable borrow of assets in the same expression.
pub fn system_audio(res: &mut Resources) {
    let queue: Vec<SoundId> = res.audio_queue.drain(..).collect();
    for id in queue {
        let sound = match id {
            SoundId::Laser => &res.sfx_laser,
            SoundId::Bump => &res.sfx_bump,
            SoundId::Lose => &res.sfx_lose,
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: 0.7,
            },
        );
    }
}
