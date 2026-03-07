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

impl TextureId {
    /// Every known variant. Update this when adding a new texture.
    pub const fn all() -> &'static [Self] {
        &[Self::Player, Self::Enemy]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundId {
    Hit,
    Blip,
}

impl SoundId {
    /// Every known variant. Update this when adding a new sound.
    pub const fn all() -> &'static [Self] {
        &[Self::Hit, Self::Blip]
    }
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

/// Holds all loaded game assets. Always valid — any loading failures are
/// stored as `warnings` (per-asset) or `critical_error` (manifest-level).
#[derive(Default)]
pub struct AssetManager {
    textures: HashMap<TextureId, Texture2D>,
    sounds: HashMap<SoundId, Sound>,
    warnings: Vec<String>,
    /// Set when the manifest itself could not be read or parsed.
    critical_error: Option<String>,
}

impl AssetManager {
    /// Infallible. Always returns a usable `AssetManager`; any manifest-level
    /// error is stored in `critical_error()` and per-asset errors in `warnings()`.
    pub async fn from_manifest(path: &str) -> Self {
        let mut assets = Self::default();

        let manifest = match Self::load_manifest(path).await {
            Ok(m) => m,
            Err(e) => {
                assets.critical_error = Some(e);
                return assets;
            }
        };

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

        // Validate that every known enum variant was supplied in the manifest.
        // Catches "added variant but forgot JSON entry" at startup.
        for id in TextureId::all() {
            if !assets.textures.contains_key(id) {
                assets.warnings.push(format!(
                    "TextureId::{id:?} is declared but missing from the asset manifest"
                ));
            }
        }
        for id in SoundId::all() {
            if !assets.sounds.contains_key(id) {
                assets.warnings.push(format!(
                    "SoundId::{id:?} is declared but missing from the asset manifest"
                ));
            }
        }

        assets
    }

    async fn load_manifest(path: &str) -> Result<AssetManifest, String> {
        let bytes = load_file(path)
            .await
            .map_err(|e| format!("Failed to read asset manifest at '{path}': {e}"))?;
        let text = String::from_utf8(bytes)
            .map_err(|e| format!("Asset manifest is not valid UTF-8: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("Invalid asset JSON: {e}"))
    }

    pub fn texture(&self, id: TextureId) -> Option<&Texture2D> {
        self.textures.get(&id)
    }

    pub fn sound(&self, id: SoundId) -> Option<&Sound> {
        self.sounds.get(&id)
    }

    /// Per-asset loading failures (file not found, corrupt data, etc.).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Manifest-level error (file missing, bad JSON). `None` on success.
    pub fn critical_error(&self) -> Option<&str> {
        self.critical_error.as_deref()
    }
}
