use macroquad::audio::{play_sound, PlaySoundParams};

use crate::resources::Resources;

/// Drain the audio queue and play each queued sound.
///
/// Uses `std::mem::take` to move the Vec out of `res` before borrowing `res.sounds`,
/// avoiding a simultaneous mutable + immutable borrow of `Resources`.
pub fn system_audio(res: &mut Resources) {
    let queue = std::mem::take(&mut res.audio_queue);

    for id in queue {
        let sound = res.sound(id);
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: 0.8,
            },
        );
    }
}
