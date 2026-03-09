// Audio stub — no asset files exist, all sounds are silent no-ops.
// To add real audio: use macroquad::audio::* and load .ogg/.wav files.

#[allow(dead_code)]
pub enum SoundEffect {
    PlayerShoot,
    EnemyExplode,
    PlayerExplode,
    SmartBomb,
    Hyperspace,
    AstronautCatch,
    BomberFire,
}

pub struct AudioSystem;

impl AudioSystem {
    pub fn new() -> Self {
        AudioSystem
    }

    #[allow(unused_variables)]
    pub fn play(&self, sfx: SoundEffect) {
        // no-op
    }
}
