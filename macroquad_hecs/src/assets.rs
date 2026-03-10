use std::collections::HashMap;

use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;

use crate::components::TextureId;
use crate::constants::ASSETS_DIR;
use crate::events::SoundId;

pub struct LoadedAssets {
    pub textures: HashMap<TextureId, Texture2D>,
    pub sounds: HashMap<SoundId, Sound>,
}

pub async fn load_all_assets() -> LoadedAssets {
    let textures = load_textures().await;
    let sounds = load_sounds().await;
    LoadedAssets { textures, sounds }
}

async fn load_textures() -> HashMap<TextureId, Texture2D> {
    async fn load(path: &str) -> Texture2D {
        let tex = load_texture(path)
            .await
            .unwrap_or_else(|e| panic!("Failed to load texture '{path}': {e}"));
        tex.set_filter(FilterMode::Nearest);
        tex
    }

    let mut map = HashMap::new();

    map.insert(
        TextureId::PlayerShip,
        load(&format!("{ASSETS_DIR}/player/player_ship.png")).await,
    );
    map.insert(
        TextureId::PlayerLaser,
        load(&format!("{ASSETS_DIR}/player/player_laser.png")).await,
    );
    map.insert(
        TextureId::EnemyShipBlack,
        load(&format!("{ASSETS_DIR}/enemies/enemy_ship_black.png")).await,
    );
    map.insert(
        TextureId::EnemyShipBlue,
        load(&format!("{ASSETS_DIR}/enemies/enemy_ship_blue.png")).await,
    );
    map.insert(
        TextureId::EnemyShipGreen,
        load(&format!("{ASSETS_DIR}/enemies/enemy_ship_green.png")).await,
    );
    map.insert(
        TextureId::EnemyShipRed,
        load(&format!("{ASSETS_DIR}/enemies/enemy_ship_red.png")).await,
    );
    map.insert(
        TextureId::EnemyLaser,
        load(&format!("{ASSETS_DIR}/enemies/enemy_laser.png")).await,
    );
    map.insert(
        TextureId::PickupLife,
        load(&format!("{ASSETS_DIR}/pickups/pickup_life.png")).await,
    );
    map.insert(
        TextureId::PickupStar,
        load(&format!("{ASSETS_DIR}/pickups/pickup_star.png")).await,
    );
    map.insert(
        TextureId::PowerupBolt,
        load(&format!("{ASSETS_DIR}/powerups/powerup_bolt.png")).await,
    );
    map.insert(
        TextureId::PowerupShield,
        load(&format!("{ASSETS_DIR}/powerups/powerup_shield.png")).await,
    );

    map
}

async fn load_sounds() -> HashMap<SoundId, Sound> {
    async fn load(path: &str) -> Sound {
        load_sound(path)
            .await
            .unwrap_or_else(|e| panic!("Failed to load sound '{path}': {e}"))
    }

    let mut map = HashMap::new();

    map.insert(
        SoundId::PlayerLaser,
        load(&format!("{ASSETS_DIR}/player/player_laser.ogg")).await,
    );
    map.insert(
        SoundId::PlayerDied,
        load(&format!("{ASSETS_DIR}/player/player_died.ogg")).await,
    );
    map.insert(
        SoundId::PlayerPowerup,
        load(&format!("{ASSETS_DIR}/player/player_powerup.ogg")).await,
    );
    map.insert(
        SoundId::EnemyLaser,
        load(&format!("{ASSETS_DIR}/enemies/enemy_laser.ogg")).await,
    );
    map.insert(
        SoundId::EnemyDestroyed,
        load(&format!("{ASSETS_DIR}/enemies/enemy_destroyed.ogg")).await,
    );
    map.insert(
        SoundId::MusicSpaceshooter,
        load(&format!("{ASSETS_DIR}/music/music_spaceshooter.ogg")).await,
    );

    map
}
