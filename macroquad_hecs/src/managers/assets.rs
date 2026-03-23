use std::collections::HashMap;

use macroquad::audio::{Sound, load_sound};
use macroquad::prelude::{FilterMode, Texture2D, load_texture};

use crate::components::TextureId;
use crate::constants::ASSETS_DIR;
use crate::events::{MusicId, SfxId};

use crate::asset_manifest::{MUSIC, SFX, TEXTURES};

pub struct Assets {
    textures: HashMap<TextureId, Texture2D>,
    sfx: HashMap<SfxId, Sound>,
    music: HashMap<MusicId, Sound>,
}

impl Assets {
    pub async fn load() -> Self {
        let mut textures = HashMap::with_capacity(TEXTURES.len());
        for &(id, file) in TEXTURES {
            textures.insert(id, Self::load_texture(file).await);
        }

        let mut sfx = HashMap::with_capacity(SFX.len());
        for &(id, file) in SFX {
            sfx.insert(id, Self::load_audio(file).await);
        }

        let mut music = HashMap::with_capacity(MUSIC.len());
        for &(id, file) in MUSIC {
            music.insert(id, Self::load_audio(file).await);
        }

        Self {
            textures,
            sfx,
            music,
        }
    }

    async fn load_texture(file: &str) -> Texture2D {
        let path = format!("{}/{}", ASSETS_DIR, file);
        let texture = load_texture(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to load texture: {}", path));
        texture.set_filter(FilterMode::Nearest);
        texture
    }

    async fn load_audio(file: &str) -> Sound {
        let path = format!("{}/{}", ASSETS_DIR, file);
        load_sound(&path)
            .await
            .unwrap_or_else(|_| panic!("Failed to load sound: {}", path))
    }

    pub fn texture(&self, id: TextureId) -> &Texture2D {
        self.textures
            .get(&id)
            .unwrap_or_else(|| panic!("Missing texture for id: {:?}", id))
    }

    pub fn sfx_bank(&self) -> &HashMap<SfxId, Sound> {
        &self.sfx
    }

    pub fn music_bank(&self) -> &HashMap<MusicId, Sound> {
        &self.music
    }
}
