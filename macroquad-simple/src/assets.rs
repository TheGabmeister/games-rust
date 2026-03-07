use std::collections::HashMap;
use macroquad::audio::{Sound, load_sound};
use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct AssetConfig {
    textures: Vec<AssetEntry>,
    sounds: Vec<AssetEntry>,
}

#[derive(Deserialize)]
struct AssetEntry {
    id: String,
    #[allow(dead_code)]
    name: String, // human-readable label, not used as a key
    path: String,
}

pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
    sounds: HashMap<String, Sound>,
}

impl AssetManager {
    pub async fn load(config_path: &str) -> Self {
        let json = load_string(config_path)
            .await
            .unwrap_or_else(|_| panic!("Failed to read {config_path}"));

        let config: AssetConfig = serde_json::from_str(&json)
            .expect("Failed to parse assets.json");

        let mut textures = HashMap::new();
        for entry in config.textures {
            let texture = load_texture(&entry.path)
                .await
                .unwrap_or_else(|_| panic!("Failed to load texture: {}", entry.path));
            texture.set_filter(FilterMode::Nearest);
            textures.insert(entry.id, texture);
        }

        let mut sounds = HashMap::new();
        for entry in config.sounds {
            let sound = load_sound(&entry.path)
                .await
                .unwrap_or_else(|_| panic!("Failed to load sound: {}", entry.path));
            sounds.insert(entry.id, sound);
        }

        Self { textures, sounds }
    }

    pub fn texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    pub fn sound(&self, name: &str) -> Option<&Sound> {
        self.sounds.get(name)
    }
}
