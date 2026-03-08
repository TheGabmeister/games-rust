use macroquad::audio::{play_sound, PlaySoundParams};

use crate::resources::{Resources, SoundId};

/// Drain the audio queue and play each pending sound.
/// Collecting first avoids a simultaneous mutable borrow of audio_queue
/// and immutable borrow of assets in the same expression.
pub fn system_audio(res: &mut Resources) {
    let queue: Vec<SoundId> = res.audio_queue.drain(..).collect();
    for id in queue {
        let sound = match id {
            SoundId::Laser => &res.sfx_laser,
            SoundId::Bump  => &res.sfx_bump,
            SoundId::Lose  => &res.sfx_lose,
        };
        play_sound(sound, PlaySoundParams { looped: false, volume: 0.7 });
    }
}
