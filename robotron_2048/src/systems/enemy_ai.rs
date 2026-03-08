use hecs::{Entity, With, World};
use macroquad::prelude::{get_frame_time, Vec2};

use crate::components::*;

const GRUNT_STEER_ACCEL: f32 = 900.0;
const HULK_STEER_ACCEL: f32 = 260.0;
const HULK_HIT_SLOW_MULTIPLIER: f32 = 0.35;

/// Enemy AI dispatch point.
pub fn system_enemy_ai(world: &mut World) {
    let dt = get_frame_time();
    let player_pos: Option<Vec2> = world
        .query::<With<(Entity, &Position), &Player>>()
        .iter()
        .next()
        .map(|(_, pos)| pos.0);

    let Some(player_pos) = player_pos else {
        return;
    };

    for (kind, vel, pos, speed, hit_slow) in
        &mut world.query::<(&EnemyKind, &mut Velocity, &Position, &Speed, &mut HitSlow)>()
    {
        hit_slow.0 = (hit_slow.0 - dt).max(0.0);

        match *kind {
            EnemyKind::Grunt => {
                let to_player = player_pos - pos.0;
                let desired = to_player.normalize_or_zero() * speed.0;
                let delta = desired - vel.0;
                let max_step = GRUNT_STEER_ACCEL * dt;
                vel.0 += delta.clamp_length_max(max_step);
            }
            EnemyKind::Hulk => {
                let slow_scale = if hit_slow.0 > 0.0 {
                    HULK_HIT_SLOW_MULTIPLIER
                } else {
                    1.0
                };
                let to_player = player_pos - pos.0;
                let desired = to_player.normalize_or_zero() * speed.0 * slow_scale;
                let delta = desired - vel.0;
                let max_step = HULK_STEER_ACCEL * dt;
                vel.0 += delta.clamp_length_max(max_step);
            }
            EnemyKind::Brain => {}
            EnemyKind::Sphereoid => {}
            EnemyKind::Enforcer => {}
            EnemyKind::Quark => {}
            EnemyKind::Tank => {}
            EnemyKind::Prog => {}
        }
    }
}
