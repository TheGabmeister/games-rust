use hecs::World;
use macroquad::prelude::*;
use ::rand::RngExt;

use crate::components::*;

const ENEMY_PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];
// Step 4 baseline: spawn Grunts only until other enemy behaviors are implemented.
const SPAWN_KINDS: [EnemyKind; 1] = [EnemyKind::Grunt];

pub fn batch_spawn_enemies(world: &mut World, n: usize) {
    let mut rng = ::rand::rng();
    for _ in 0..n {
        let pos = vec2(rng.random_range(0.0..800.0), rng.random_range(0.0..600.0));
        let kind = SPAWN_KINDS[rng.random_range(0..SPAWN_KINDS.len())];
        spawn_enemy(world, kind, pos);
    }
}

pub fn spawn_enemy(world: &mut World, kind: EnemyKind, pos: Vec2) {
    let profile = enemy_profile(kind);
    let tint = ENEMY_PALETTE[(kind as usize) % ENEMY_PALETTE.len()];

    world.spawn((
        Position(pos),
        Velocity(Vec2::ZERO),
        Speed(profile.speed),
        Health(profile.health),
        kind,
        Invulnerable(matches!(kind, EnemyKind::Hulk)),
        FireCooldown {
            remaining: profile.fire_period,
            period: profile.fire_period,
        },
        SpawnCooldown {
            remaining: profile.spawn_period,
            period: profile.spawn_period,
        },
        Sprite {
            texture: TextureId::EnemyBlack,
            tint,
        },
        DrawLayer(LAYER_ENEMY),
        Collider::Circle {
            radius: profile.radius,
        },
    ));
}

/// Enemy spawn dispatch point.
/// This currently only advances spawn cooldowns; child spawning comes next.
pub fn system_enemy_spawn(world: &mut World) {
    let dt = get_frame_time();

    for (kind, cooldown) in &mut world.query::<(&EnemyKind, &mut SpawnCooldown)>() {
        cooldown.remaining = (cooldown.remaining - dt).max(0.0);

        match *kind {
            EnemyKind::Sphereoid | EnemyKind::Quark => {
                if cooldown.remaining <= 0.0 && cooldown.period > 0.0 {
                    cooldown.remaining = cooldown.period;
                }
            }
            EnemyKind::Grunt
            | EnemyKind::Hulk
            | EnemyKind::Brain
            | EnemyKind::Enforcer
            | EnemyKind::Tank
            | EnemyKind::Prog => {}
        }
    }
}

struct EnemyProfile {
    speed: f32,
    health: i32,
    radius: f32,
    fire_period: f32,
    spawn_period: f32,
}

fn enemy_profile(kind: EnemyKind) -> EnemyProfile {
    match kind {
        EnemyKind::Grunt => EnemyProfile {
            speed: 140.0,
            health: 1,
            radius: 16.0,
            fire_period: 0.0,
            spawn_period: 0.0,
        },
        EnemyKind::Hulk => EnemyProfile {
            speed: 80.0,
            health: 4,
            radius: 20.0,
            fire_period: 0.0,
            spawn_period: 0.0,
        },
        EnemyKind::Brain => EnemyProfile {
            speed: 120.0,
            health: 1,
            radius: 16.0,
            fire_period: 3.5,
            spawn_period: 0.0,
        },
        EnemyKind::Sphereoid => EnemyProfile {
            speed: 30.0,
            health: 2,
            radius: 22.0,
            fire_period: 0.0,
            spawn_period: 4.0,
        },
        EnemyKind::Enforcer => EnemyProfile {
            speed: 110.0,
            health: 1,
            radius: 16.0,
            fire_period: 1.5,
            spawn_period: 0.0,
        },
        EnemyKind::Quark => EnemyProfile {
            speed: 25.0,
            health: 3,
            radius: 22.0,
            fire_period: 0.0,
            spawn_period: 5.0,
        },
        EnemyKind::Tank => EnemyProfile {
            speed: 70.0,
            health: 2,
            radius: 18.0,
            fire_period: 2.0,
            spawn_period: 0.0,
        },
        EnemyKind::Prog => EnemyProfile {
            speed: 180.0,
            health: 1,
            radius: 14.0,
            fire_period: 0.0,
            spawn_period: 0.0,
        },
    }
}
