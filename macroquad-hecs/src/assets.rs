use std::collections::HashMap;

use macroquad::audio::{Sound, load_sound};
use macroquad::file::load_file;
use macroquad::prelude::{FilterMode, Texture2D, load_texture};
use serde::Deserialize;

pub const DEFAULT_ASSET_MANIFEST: &str = "assets/assets.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureId {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundId {
    Hit,
    Blip,
}

#[derive(Debug, Deserialize)]
struct AssetManifest {
    #[serde(default)]
    textures: Vec<AssetEntry<TextureId>>,
    #[serde(default)]
    audio: Vec<AssetEntry<SoundId>>,
}

#[derive(Debug, Deserialize)]
struct AssetEntry<T> {
    #[serde(alias = "name")]
    id: T,
    path: String,
}

#[derive(Default)]
pub struct AssetManager {
    textures: HashMap<TextureId, Texture2D>,
    sounds: HashMap<SoundId, Sound>,
    warnings: Vec<String>,
}

impl AssetManager {
    pub async fn from_manifest(path: &str) -> Result<Self, String> {
        let bytes = load_file(path)
            .await
            .map_err(|e| format!("Failed to read asset manifest at '{path}': {e}"))?;
        let text = String::from_utf8(bytes)
            .map_err(|e| format!("Asset manifest is not valid UTF-8: {e}"))?;
        let manifest: AssetManifest =
            serde_json::from_str(&text).map_err(|e| format!("Invalid asset JSON: {e}"))?;

        let mut assets = Self::default();

        for entry in manifest.textures {
            match load_texture(&entry.path).await {
                Ok(texture) => {
                    texture.set_filter(FilterMode::Nearest);
                    assets.textures.insert(entry.id, texture);
                }
                Err(e) => assets.warnings.push(format!(
                    "Texture '{:?}' failed to load from '{}': {e}",
                    entry.id, entry.path
                )),
            }
        }

        for entry in manifest.audio {
            match load_sound(&entry.path).await {
                Ok(sound) => {
                    assets.sounds.insert(entry.id, sound);
                }
                Err(e) => assets.warnings.push(format!(
                    "Audio '{:?}' failed to load from '{}': {e}",
                    entry.id, entry.path
                )),
            }
        }

        Ok(assets)
    }

    pub fn texture(&self, id: TextureId) -> Option<&Texture2D> {
        self.textures.get(&id)
    }

    pub fn sound(&self, id: SoundId) -> Option<&Sound> {
        self.sounds.get(&id)
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
