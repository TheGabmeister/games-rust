use hecs::World;
use macroquad::prelude::get_frame_time;

use crate::components::*;

/// Enemy attack dispatch point.
/// This currently only advances cooldowns; firing behavior will be implemented next.
pub fn system_enemy_attack(world: &mut World) {
    let dt = get_frame_time();

    for (kind, cooldown) in &mut world.query::<(&EnemyKind, &mut FireCooldown)>() {
        cooldown.remaining = (cooldown.remaining - dt).max(0.0);

        match *kind {
            EnemyKind::Enforcer | EnemyKind::Tank | EnemyKind::Brain => {}
            EnemyKind::Grunt
            | EnemyKind::Hulk
            | EnemyKind::Sphereoid
            | EnemyKind::Quark
            | EnemyKind::Prog => {}
        }
    }
}
