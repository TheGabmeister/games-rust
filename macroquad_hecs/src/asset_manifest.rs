use crate::components::TextureId;
use crate::events::{MusicId, SfxId};

pub const TEXTURES: &[(TextureId, &str)] = &[
    (TextureId::PlayerShip, "player/player_ship.png"),
    (TextureId::PlayerShipLeft, "player/player_ship_left.png"),
    (TextureId::PlayerShipRight, "player/player_ship_right.png"),
    (TextureId::PlayerLaser, "player/player_laser.png"),
    (TextureId::EnemyLaser, "enemies/enemy_laser.png"),
    (TextureId::EnemyShipBlack, "enemies/enemy_ship_black.png"),
    (TextureId::EnemyShipBlue, "enemies/enemy_ship_blue.png"),
    (TextureId::EnemyShipGreen, "enemies/enemy_ship_green.png"),
    (TextureId::EnemyShipRed, "enemies/enemy_ship_red.png"),
    (TextureId::PickupLife, "pickups/pickup_life.png"),
    (TextureId::PickupStar, "pickups/pickup_star.png"),
    (TextureId::PowerupBolt, "powerups/powerup_bolt.png"),
    (TextureId::PowerupShield, "powerups/powerup_shield.png"),
];

pub const SFX: &[(SfxId, &str)] = &[
    (SfxId::PlayerLaser, "player/player_laser.ogg"),
    (SfxId::PlayerDied, "player/player_died.ogg"),
    (SfxId::PlayerPowerup, "player/player_powerup.ogg"),
    (SfxId::EnemyLaser, "enemies/enemy_laser.ogg"),
    (SfxId::EnemyDestroyed, "enemies/enemy_destroyed.ogg"),
];

pub const MUSIC: &[(MusicId, &str)] = &[(MusicId::Spaceshooter, "music/music_spaceshooter.ogg")];
