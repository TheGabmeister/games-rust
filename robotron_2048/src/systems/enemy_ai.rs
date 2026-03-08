use hecs::{Entity, With, World};
use macroquad::prelude::{get_frame_time, Vec2};

use crate::components::*;

const GRUNT_STEER_ACCEL: f32 = 900.0;

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

    for (kind, vel, pos, speed) in &mut world.query::<(&EnemyKind, &mut Velocity, &Position, &Speed)>() {
        match *kind {
            EnemyKind::Grunt => {
                let to_player = player_pos - pos.0;
                let desired = to_player.normalize_or_zero() * speed.0;
                let delta = desired - vel.0;
                let max_step = GRUNT_STEER_ACCEL * dt;
                vel.0 += delta.clamp_length_max(max_step);
            }
            EnemyKind::Hulk => {}
            EnemyKind::Brain => {}
            EnemyKind::Sphereoid => {}
            EnemyKind::Enforcer => {}
            EnemyKind::Quark => {}
            EnemyKind::Tank => {}
            EnemyKind::Prog => {}
        }
    }
}
