use macroquad::audio::{Sound, load_sound_from_bytes, play_sound_once};

use crate::domain::GameEvent;

pub struct AudioSystem {
    shot: Option<Sound>,
    hit: Option<Sound>,
    enemy_kill: Option<Sound>,
    player_death: Option<Sound>,
    round_clear: Option<Sound>,
    extra_life: Option<Sound>,
}

impl AudioSystem {
    pub async fn new() -> Self {
        let shot = load_tone(940.0, 0.05, 0.24).await;
        let hit = load_tone(420.0, 0.06, 0.22).await;
        let enemy_kill = load_tone(660.0, 0.08, 0.3).await;
        let player_death = load_tone(180.0, 0.22, 0.36).await;
        let round_clear = load_tone(520.0, 0.18, 0.28).await;
        let extra_life = load_tone(860.0, 0.18, 0.26).await;

        Self {
            shot,
            hit,
            enemy_kill,
            player_death,
            round_clear,
            extra_life,
        }
    }

    pub fn consume(&self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::Shot => play_opt(&self.shot),
                GameEvent::Hit => play_opt(&self.hit),
                GameEvent::EnemyKilled(_) => play_opt(&self.enemy_kill),
                GameEvent::PlayerDied => play_opt(&self.player_death),
                GameEvent::RoundCleared(_) => play_opt(&self.round_clear),
                GameEvent::ExtraLife => play_opt(&self.extra_life),
            }
        }
    }
}

fn play_opt(sound: &Option<Sound>) {
    if let Some(sound) = sound {
        play_sound_once(sound);
    }
}

async fn load_tone(freq: f32, duration: f32, volume: f32) -> Option<Sound> {
    let bytes = build_tone_wav(freq, duration, volume);
    load_sound_from_bytes(&bytes).await.ok()
}

fn build_tone_wav(freq: f32, duration: f32, volume: f32) -> Vec<u8> {
    let sample_rate = 22_050u32;
    let sample_count = (sample_rate as f32 * duration).max(1.0) as usize;
    let mut samples = Vec::with_capacity(sample_count);

    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        let phase = std::f32::consts::TAU * freq * t;
        let attack = (i as f32 / (sample_count as f32 * 0.08)).min(1.0);
        let decay = ((sample_count - i) as f32 / (sample_count as f32 * 0.2)).min(1.0);
        let env = attack.min(decay);
        let value = (phase.sin() * volume * env * i16::MAX as f32) as i16;
        samples.push(value);
    }

    let data_bytes_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_bytes_len as usize);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
}
