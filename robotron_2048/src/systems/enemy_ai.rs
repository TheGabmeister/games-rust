use ::rand::RngExt;
use hecs::{With, World};
use macroquad::prelude::Vec2;

use crate::components::*;

/// Enemy AI dispatch point based on capability components.
pub fn system_enemy_ai(world: &mut World, rng: &mut ::rand::rngs::ThreadRng, dt: f32) {
    let player_pos = world
        .query::<With<&Position, &Player>>()
        .iter()
        .next()
        .map(|pos| pos.0);

    let Some(player_pos) = player_pos else {
        return;
    };

    for (vel, pos, speed, chase, maybe_hit_slow) in &mut world.query::<(
        &mut Velocity,
        &Position,
        &Speed,
        &mut Chase,
        Option<&mut HitSlow>,
    )>() {
        let mut speed_scale = 1.0;
        if let Some(hit_slow) = maybe_hit_slow {
            hit_slow.0 = (hit_slow.0 - dt).max(0.0);
            if hit_slow.0 > 0.0 {
                speed_scale = chase.hit_slow_multiplier;
            }
        }

        let to_player = player_pos - pos.0;
        let forward = to_player.normalize_or_zero();

        // Strafe direction is held for a random interval (0.4–0.9 s) then flipped,
        // producing real strafing arcs instead of per-step jitter.
        let strafe_sign = if chase.strafe_weight > 0.0 {
            chase.strafe_timer -= dt;
            if chase.strafe_timer <= 0.0 {
                chase.strafe_sign = if rng.random_range(0.0..1.0_f32) < 0.5 {
                    -1.0
                } else {
                    1.0
                };
                chase.strafe_timer = rng.random_range(0.4_f32..0.9_f32);
            }
            chase.strafe_sign
        } else {
            0.0
        };

        let strafe = Vec2::new(-forward.y, forward.x) * strafe_sign;
        let jitter = if chase.jitter_weight > 0.0 {
            Vec2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)).normalize_or_zero()
        } else {
            Vec2::ZERO
        };

        let desired_dir = (forward * chase.forward_weight
            + strafe * chase.strafe_weight
            + jitter * chase.jitter_weight)
            .normalize_or_zero();
        let desired = desired_dir * speed.0 * speed_scale;
        let delta = desired - vel.0;
        let max_step = chase.steer_accel * dt;
        vel.0 += delta.clamp_length_max(max_step);
    }
}
