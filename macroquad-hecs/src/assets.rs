#![windows_subsystem = "windows"] // Stops console window from showing when running in Windows
#![cfg_attr(debug_assertions, allow(unused))] // Warn user of unused code during Release builds.

use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;

const ASSETS_DIR: &str = "assets";

pub struct Assets {
    pub player_ship:        Texture2D,
    pub sfx_laser:          Sound,
    pub sfx_bump:           Sound,
    pub sfx_lose:           Sound,
    pub music_spaceshooter: Sound,
}

impl Assets {
    pub async fn load() -> Self {
        Self {
            player_ship:        Self::texture("player_ship.png").await,
            sfx_laser:          Self::sound("sfx_laser1.ogg").await,
            sfx_bump:           Self::sound("sfx_bump.ogg").await,
            sfx_lose:           Self::sound("sfx_lose.ogg").await,
            music_spaceshooter: Self::sound("music_spaceshooter.ogg").await,
        }
    }

    async fn texture(file: &str) -> Texture2D {
        let path = format!("{}/{}", ASSETS_DIR, file);
        load_texture(&path).await
            .unwrap_or_else(|_| panic!("Failed to load texture: {}", path))
    }

    async fn sound(file: &str) -> Sound {
        let path = format!("{}/{}", ASSETS_DIR, file);
        load_sound(&path).await
            .unwrap_or_else(|_| panic!("Failed to load sound: {}", path))
    }
}
