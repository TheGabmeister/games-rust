use ::rand::RngExt;
use hecs::{Entity, With, World};
use macroquad::prelude::*;

use crate::components::*;

/// Enemy attack dispatch point based on RangedAttack capability.
pub fn system_enemy_attack(world: &mut World, dt: f32) {
    let player_pos = world
        .query::<With<&Position, &Player>>()
        .iter()
        .next()
        .map(|pos| pos.0);
    let Some(player_pos) = player_pos else {
        return;
    };

    let mut fire_events: Vec<(Entity, Vec2, RangedAttack)> = Vec::new();

    for (entity, pos, attack) in &mut world.query::<(Entity, &Position, &mut RangedAttack)>() {
        attack.remaining = (attack.remaining - dt).max(0.0);
        if attack.period <= 0.0 || attack.remaining > 0.0 {
            continue;
        }

        fire_events.push((entity, pos.0, *attack));
        attack.remaining = attack.period;
    }

    let mut rng = ::rand::rng();
    for (owner, origin, attack) in fire_events {
        let to_player = player_pos - origin;
        let base = to_player.normalize_or_zero();
        if base == Vec2::ZERO {
            continue;
        }

        let base_angle = base.y.atan2(base.x);
        let jitter = rng.random_range(-attack.aim_jitter_rad..attack.aim_jitter_rad);
        let angle = base_angle + jitter;
        let dir = vec2(angle.cos(), angle.sin());

        world.spawn((
            Position(origin),
            Velocity(dir * attack.projectile_speed),
            Lifetime(attack.projectile_lifetime),
            Projectile {
                owner,
                faction: Faction::Enemy,
                kind: attack.projectile_kind,
                damage: attack.projectile_damage,
            },
            Collider::Circle {
                radius: attack.projectile_radius,
            },
            Sprite {
                texture: attack.projectile_texture,
                tint: attack.projectile_tint,
            },
            DrawLayer(LAYER_PROJECTILE),
        ));
    }
}
