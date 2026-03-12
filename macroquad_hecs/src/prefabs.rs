use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::constants::*;

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

pub fn spawn_player(world: &mut World) -> Entity {
    world.spawn((
        Transform::at(PLAYER_START_X, PLAYER_START_Y),
        Velocity::zero(),
        Sprite::new(TextureId::PlayerShip),
        BoxCollider::new(32.0, 32.0),
        CollisionLayer {
            member: LAYER_PLAYER,
            mask: LAYER_ENEMY | LAYER_ENEMY_BULLET | LAYER_PICKUP,
        },
        Player,
        FireTimer::new(PLAYER_FIRE_RATE),
        DrawLayer(DRAW_PLAYER),
    ))
}

// ---------------------------------------------------------------------------
// Enemies
// ---------------------------------------------------------------------------

pub fn spawn_enemy(world: &mut World, kind: EnemyKind, pos: Vec2) -> Entity {
    let (texture, speed, score) = match kind {
        EnemyKind::Black => (
            TextureId::EnemyShipBlack,
            ENEMY_SPEED_BLACK,
            SCORE_ENEMY_BLACK,
        ),
        EnemyKind::Blue => (TextureId::EnemyShipBlue, ENEMY_SPEED_BLUE, SCORE_ENEMY_BLUE),
        EnemyKind::Green => (
            TextureId::EnemyShipGreen,
            ENEMY_SPEED_GREEN,
            SCORE_ENEMY_GREEN,
        ),
        EnemyKind::Red => (TextureId::EnemyShipRed, ENEMY_SPEED_RED, SCORE_ENEMY_RED),
    };

    world.spawn((
        Transform {
            pos,
            rot: std::f32::consts::PI,
        }, // facing down
        Velocity::new(0.0, speed),
        Sprite::new(texture),
        BoxCollider::new(32.0, 32.0),
        CollisionLayer {
            member: LAYER_ENEMY,
            mask: LAYER_PLAYER | LAYER_PLAYER_BULLET,
        },
        Enemy { kind },
        FireTimer::new(ENEMY_FIRE_RATE),
        ScoreValue(score),
        DrawLayer(DRAW_ENEMY),
    ))
}

// ---------------------------------------------------------------------------
// Bullets
// ---------------------------------------------------------------------------

pub fn spawn_player_bullet(world: &mut World, pos: Vec2, speed: f32) -> Entity {
    world.spawn((
        Transform::at(pos.x, pos.y),
        Velocity::new(0.0, -speed),
        Sprite::new(TextureId::PlayerLaser),
        BoxCollider::new(6.0, 16.0),
        CollisionLayer {
            member: LAYER_PLAYER_BULLET,
            mask: LAYER_ENEMY,
        },
        Projectile {
            owner: ProjectileOwner::Player,
        },
        Lifetime::new(BULLET_LIFETIME),
        DrawLayer(DRAW_BULLET),
    ))
}

pub fn spawn_enemy_bullet(world: &mut World, pos: Vec2, speed: f32) -> Entity {
    world.spawn((
        Transform::at(pos.x, pos.y),
        Velocity::new(0.0, speed),
        Sprite::new(TextureId::EnemyLaser),
        BoxCollider::new(6.0, 16.0),
        CollisionLayer {
            member: LAYER_ENEMY_BULLET,
            mask: LAYER_PLAYER,
        },
        Projectile {
            owner: ProjectileOwner::Enemy,
        },
        Lifetime::new(BULLET_LIFETIME),
        DrawLayer(DRAW_BULLET),
    ))
}

pub fn spawn_pickup(world: &mut World, kind: PickupKind, pos: Vec2) -> Entity {
    let texture = match kind {
        PickupKind::Life => TextureId::PickupLife,
        PickupKind::Star => TextureId::PickupStar,
    };

    world.spawn((
        Transform::at(pos.x, pos.y),
        Sprite::new(texture),
        CircleCollider::new(12.0),
        CollisionLayer {
            member: LAYER_PICKUP,
            mask: LAYER_PLAYER,
        },
        Pickup { kind },
        DrawLayer(DRAW_PICKUP),
    ))
}

pub fn spawn_powerup(world: &mut World, effect: PowerupEffect, pos: Vec2) -> Entity {
    let texture = match effect {
        PowerupEffect::Bolt => TextureId::PowerupBolt,
        PowerupEffect::Shield => TextureId::PowerupShield,
    };

    world.spawn((
        Transform::at(pos.x, pos.y),
        Sprite::new(texture),
        CircleCollider::new(12.0),
        CollisionLayer {
            member: LAYER_PICKUP,
            mask: LAYER_PLAYER,
        },
        ActivePowerup {
            effect,
            duration: 5.0,
        },
        DrawLayer(DRAW_PICKUP),
    ))
}

pub fn spawn_obstacle(world: &mut World, pos: Vec2, size: Vec2) -> Entity {
    world.spawn((
        Transform::at(pos.x, pos.y),
        BoxCollider::new(size.x, size.y),
        CollisionLayer {
            member: LAYER_ENEMY, // treated as enemy-side for collision
            mask: LAYER_PLAYER | LAYER_PLAYER_BULLET,
        },
        DrawLayer(DRAW_BACKGROUND),
    ))
}
