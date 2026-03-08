use ::rand::RngExt;
use hecs::{EntityBuilder, With, World};
use macroquad::prelude::*;

use crate::components::*;
use crate::resources::{WaveDirector, WaveSpawnRequest};

const ENEMY_PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];
const HULK_HIT_SLOW_SECONDS: f32 = 0.35;
const HULK_KNOCKBACK_SPEED: f32 = 180.0;
const MIN_SPAWN_DISTANCE_FROM_PLAYER: f32 = 140.0;
const SPAWN_MARGIN: f32 = 24.0;
const ENFORCER_EMBRYO_RADIUS: f32 = 9.0;

pub fn system_wave_director(
    world: &mut World,
    wave_director: &mut WaveDirector,
    rng: &mut ::rand::rngs::ThreadRng,
) {
    let Some(request) = wave_director.consume_spawn_request() else {
        return;
    };

    spawn_wave(world, request, rng);
}

fn spawn_wave(world: &mut World, request: WaveSpawnRequest, rng: &mut ::rand::rngs::ThreadRng) {
    let player_pos = world
        .query::<With<&Position, &Player>>()
        .iter()
        .next()
        .map(|pos| pos.0);

    for entry in request.definition.entries {
        let count = scaled_wave_count(entry.count, request.difficulty_cycle);
        for _ in 0..count {
            let mut spawn_pos = random_arena_position(rng);
            for _ in 0..8 {
                if player_pos.is_none_or(|player| {
                    player.distance(spawn_pos) >= MIN_SPAWN_DISTANCE_FROM_PLAYER
                }) {
                    break;
                }
                spawn_pos = random_arena_position(rng);
            }
            spawn_enemy(world, entry.kind, spawn_pos);
        }
    }
}

fn scaled_wave_count(base: usize, difficulty_cycle: usize) -> usize {
    base + difficulty_cycle * base.max(1).div_ceil(2)
}

fn random_arena_position(rng: &mut ::rand::rngs::ThreadRng) -> Vec2 {
    let min_x = SPAWN_MARGIN;
    let max_x = (screen_width() - SPAWN_MARGIN).max(min_x + 1.0);
    let min_y = SPAWN_MARGIN;
    let max_y = (screen_height() - SPAWN_MARGIN).max(min_y + 1.0);
    vec2(
        rng.random_range(min_x..max_x),
        rng.random_range(min_y..max_y),
    )
}

fn clamp_to_arena(pos: Vec2, radius: f32) -> Vec2 {
    let min_x = radius + SPAWN_MARGIN;
    let max_x = (screen_width() - radius - SPAWN_MARGIN).max(min_x);
    let min_y = radius + SPAWN_MARGIN;
    let max_y = (screen_height() - radius - SPAWN_MARGIN).max(min_y);
    vec2(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y))
}

pub fn spawn_enemy(world: &mut World, kind: EnemyKind, pos: Vec2) {
    let profile = enemy_profile(kind);
    let tint = ENEMY_PALETTE[(kind as usize) % ENEMY_PALETTE.len()];
    let clamped_pos = clamp_to_arena(pos, profile.radius);
    let mut entity = EntityBuilder::new();

    entity.add(Position(clamped_pos));
    entity.add(Velocity(Vec2::ZERO));
    entity.add(Speed(profile.speed));
    if let Some(h) = profile.health {
        entity.add(Health(h));
    }
    entity.add(kind);
    entity.add(Sprite {
        texture: TextureId::EnemyBlack,
        tint,
    });
    entity.add(DrawLayer(LAYER_ENEMY));
    entity.add(Collider::Circle {
        radius: profile.radius,
    });

    if profile.invulnerable {
        entity.add(Invulnerable);
    }
    if let Some(chase) = profile.chase {
        entity.add(chase);
    }
    if profile.uses_hit_slow {
        entity.add(HitSlow(0.0));
    }
    if let Some(hit_reaction) = profile.hit_reaction {
        entity.add(hit_reaction);
    }
    if let Some(attack) = profile.ranged_attack {
        entity.add(attack);
    }
    if let Some(spawner) = profile.spawner {
        entity.add(spawner);
    }
    if let Some(contact_damage) = profile.contact_damage {
        entity.add(contact_damage);
    }
    if profile.counts_for_wave_clear {
        entity.add(WaveClearTarget);
    }

    world.spawn(entity.build());
}

fn spawn_embryo(world: &mut World, target_kind: EnemyKind, pos: Vec2, mature_time: f32) {
    let clamped_pos = clamp_to_arena(pos, ENFORCER_EMBRYO_RADIUS);

    world.spawn((
        Position(clamped_pos),
        Velocity(Vec2::ZERO),
        Health(1),
        target_kind,
        Growth {
            remaining: mature_time,
            target_kind,
        },
        Sprite {
            texture: TextureId::EnemyBlack,
            tint: PINK,
        },
        DrawLayer(LAYER_ENEMY),
        Collider::Circle {
            radius: ENFORCER_EMBRYO_RADIUS,
        },
        WaveClearTarget,
    ));
}

/// Tick spawner components and emit child enemies for entities that can spawn.
pub fn system_enemy_spawn(world: &mut World, rng: &mut ::rand::rngs::ThreadRng, dt: f32) {
    let mut spawn_events: Vec<(EnemyKind, Vec2, f32)> = Vec::new();

    for (pos, spawner) in &mut world.query::<(&Position, &mut Spawner)>() {
        spawner.remaining = (spawner.remaining - dt).max(0.0);
        if spawner.period <= 0.0 || spawner.remaining > 0.0 {
            continue;
        }

        for _ in 0..spawner.burst_count {
            let angle = rng.random_range(0.0..(2.0 * std::f32::consts::PI));
            let distance = rng.random_range(0.0..spawner.spawn_radius.max(0.0));
            let offset = vec2(angle.cos(), angle.sin()) * distance;
            spawn_events.push((
                spawner.spawn_kind,
                pos.0 + offset,
                spawner.embryo_mature_time,
            ));
        }

        spawner.remaining = spawner.period;
    }

    for (kind, spawn_pos, embryo_mature_time) in spawn_events {
        if embryo_mature_time > 0.0 {
            spawn_embryo(world, kind, spawn_pos, embryo_mature_time);
        } else {
            spawn_enemy(world, kind, spawn_pos);
        }
    }
}

/// Tick embryo growth and replace matured embryos with their final enemy form.
pub fn system_enemy_maturation(world: &mut World, cmd: &mut hecs::CommandBuffer, dt: f32) {
    let mut mature_events: Vec<(hecs::Entity, EnemyKind, Vec2)> = Vec::new();

    for (entity, pos, growth) in &mut world.query::<(hecs::Entity, &Position, &mut Growth)>() {
        growth.remaining -= dt;
        if growth.remaining <= 0.0 {
            mature_events.push((entity, growth.target_kind, pos.0));
        }
    }

    for (entity, target_kind, pos) in mature_events {
        cmd.despawn(entity);
        spawn_enemy(world, target_kind, pos);
    }
}

#[derive(Clone, Copy)]
struct EnemyProfile {
    speed: f32,
    health: Option<i32>,
    radius: f32,
    invulnerable: bool,
    uses_hit_slow: bool,
    counts_for_wave_clear: bool,
    chase: Option<Chase>,
    ranged_attack: Option<RangedAttack>,
    spawner: Option<Spawner>,
    contact_damage: Option<ContactDamage>,
    hit_reaction: Option<HitReaction>,
}

fn enemy_profile(kind: EnemyKind) -> EnemyProfile {
    match kind {
        EnemyKind::Grunt => EnemyProfile {
            speed: 140.0,
            health: Some(1),
            radius: 16.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: Some(Chase {
                steer_accel: 900.0,
                forward_weight: 1.0,
                strafe_weight: 0.0,
                jitter_weight: 0.0,
                hit_slow_multiplier: 1.0,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: None,
            spawner: None,
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Hulk => EnemyProfile {
            speed: 80.0,
            health: None,
            radius: 20.0,
            invulnerable: true,
            uses_hit_slow: true,
            counts_for_wave_clear: false,
            chase: Some(Chase {
                steer_accel: 260.0,
                forward_weight: 1.0,
                strafe_weight: 0.0,
                jitter_weight: 0.0,
                hit_slow_multiplier: 0.35,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: None,
            spawner: None,
            contact_damage: None,
            hit_reaction: Some(HitReaction {
                hit_slow_seconds: HULK_HIT_SLOW_SECONDS,
                knockback_speed: HULK_KNOCKBACK_SPEED,
            }),
        },
        EnemyKind::Brain => EnemyProfile {
            speed: 120.0,
            health: Some(1),
            radius: 16.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: Some(Chase {
                steer_accel: 600.0,
                forward_weight: 1.0,
                strafe_weight: 0.0,
                jitter_weight: 0.15,
                hit_slow_multiplier: 1.0,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: None,
            spawner: None,
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Sphereoid => EnemyProfile {
            speed: 30.0,
            health: Some(2),
            radius: 22.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: None,
            ranged_attack: None,
            spawner: Some(Spawner {
                remaining: 4.0,
                period: 4.0,
                spawn_kind: EnemyKind::Enforcer,
                burst_count: 1,
                spawn_radius: 90.0,
                embryo_mature_time: 1.7,
            }),
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Enforcer => EnemyProfile {
            speed: 110.0,
            health: Some(1),
            radius: 16.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: Some(Chase {
                steer_accel: 520.0,
                forward_weight: 0.45,
                strafe_weight: 0.85,
                jitter_weight: 0.55,
                hit_slow_multiplier: 1.0,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: Some(RangedAttack {
                remaining: 1.5,
                period: 1.5,
                projectile_kind: ProjectileKind::EnforcerSpark,
                projectile_speed: 260.0,
                projectile_lifetime: 2.2,
                projectile_radius: 4.0,
                projectile_damage: 1,
                projectile_texture: TextureId::PlayerLaser,
                projectile_tint: ORANGE,
                aim_jitter_rad: 0.35,
            }),
            spawner: None,
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Quark => EnemyProfile {
            speed: 25.0,
            health: Some(3),
            radius: 22.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: None,
            ranged_attack: None,
            spawner: Some(Spawner {
                remaining: 5.0,
                period: 5.0,
                spawn_kind: EnemyKind::Tank,
                burst_count: 1,
                spawn_radius: 95.0,
                embryo_mature_time: 0.0,
            }),
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Tank => EnemyProfile {
            speed: 70.0,
            health: Some(2),
            radius: 18.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: Some(Chase {
                steer_accel: 360.0,
                forward_weight: 1.0,
                strafe_weight: 0.0,
                jitter_weight: 0.1,
                hit_slow_multiplier: 1.0,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: Some(RangedAttack {
                remaining: 2.0,
                period: 2.0,
                projectile_kind: ProjectileKind::TankShell,
                projectile_speed: 210.0,
                projectile_lifetime: 3.0,
                projectile_radius: 6.0,
                projectile_damage: 1,
                projectile_texture: TextureId::PlayerLaser,
                projectile_tint: YELLOW,
                aim_jitter_rad: 0.12,
            }),
            spawner: None,
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
        EnemyKind::Prog => EnemyProfile {
            speed: 180.0,
            health: Some(1),
            radius: 14.0,
            invulnerable: false,
            uses_hit_slow: false,
            counts_for_wave_clear: true,
            chase: Some(Chase {
                steer_accel: 1100.0,
                forward_weight: 1.0,
                strafe_weight: 0.0,
                jitter_weight: 0.15,
                hit_slow_multiplier: 1.0,
                strafe_sign: 0.0,
                strafe_timer: 0.0,
            }),
            ranged_attack: None,
            spawner: None,
            contact_damage: Some(ContactDamage { damage: 1 }),
            hit_reaction: None,
        },
    }
}
