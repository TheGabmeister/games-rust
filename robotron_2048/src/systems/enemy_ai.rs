use hecs::World;

use crate::components::*;

/// Enemy AI dispatch point.
/// Behavior is intentionally a no-op for now; per-kind logic will be added next.
pub fn system_enemy_ai(world: &mut World) {
    for (kind, vel, pos, speed) in &mut world.query::<(&EnemyKind, &mut Velocity, &Position, &Speed)>() {
        match *kind {
            EnemyKind::Grunt => {}
            EnemyKind::Hulk => {}
            EnemyKind::Brain => {}
            EnemyKind::Sphereoid => {}
            EnemyKind::Enforcer => {}
            EnemyKind::Quark => {}
            EnemyKind::Tank => {}
            EnemyKind::Prog => {}
        }

        let _ = (vel, pos, speed);
    }
}
