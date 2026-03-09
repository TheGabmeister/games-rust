use ::rand::Rng;
use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    Collider, Enemy, EnemyKind, EnemyMode, PathFollower, PathKind, RenderablePrimitive,
    TractorBeamPhase, TractorBeamState, Transform,
};
use crate::constants::{PLAYER_Y, SCREEN_WIDTH};
use crate::events::GameEvent;
use crate::patterns::{capture_approach_path, dive_path, return_path, sample_path};
use crate::resources::{GameMode, Resources, StageType};
use crate::rules::can_enemy_fire;

use super::helpers::{enemy_appearance, player_position, set_path};
use super::player::find_player_entity;

pub(super) fn schedule_enemy_dives(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode != GameMode::Playing || resources.stage.stage_type != StageType::Normal {
        return;
    }

    resources.difficulty.dive_timer -= dt;
    if resources.difficulty.dive_timer > 0.0 {
        return;
    }

    let mut active_divers = 0usize;
    for enemy in world.query::<&Enemy>().iter() {
        if matches!(
            enemy.mode,
            EnemyMode::Diving | EnemyMode::Capturing | EnemyMode::Returning
        ) {
            active_divers += 1;
        }
    }

    if active_divers >= resources.difficulty.max_divers {
        resources.difficulty.dive_timer = resources.difficulty.dive_interval * 0.5;
        return;
    }

    let player_x = player_position(world)
        .map(|p| p.x)
        .unwrap_or(SCREEN_WIDTH * 0.5);

    let candidates: Vec<Entity> = world
        .query::<(Entity, &Enemy)>()
        .iter()
        .filter_map(|(entity, enemy)| {
            if enemy.mode == EnemyMode::Formed {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        resources.difficulty.dive_timer = resources.difficulty.dive_interval;
        return;
    }

    let chosen = candidates[resources.rng.rng.gen_range(0..candidates.len())];

    let transform = if let Ok(transform) = world.get::<&Transform>(chosen) {
        transform.pos
    } else {
        resources.difficulty.dive_timer = resources.difficulty.dive_interval;
        return;
    };

    let slot_target = if let Ok(slot) = world.get::<&crate::components::FormationSlot>(chosen) {
        slot.target
    } else {
        transform
    };

    let mut capture_attack = false;
    if let Ok(enemy) = world.get::<&Enemy>(chosen) {
        if enemy.kind == EnemyKind::BossGalaga
            && !enemy.carrying_player
            && !resources.player.dual_active
            && find_player_entity(world).is_some()
        {
            capture_attack = resources.rng.rng.gen_bool(0.35);
        }
    }

    if let Ok(mut enemy) = world.get::<&mut Enemy>(chosen) {
        if capture_attack {
            enemy.mode = EnemyMode::Capturing;
        } else {
            enemy.mode = EnemyMode::Diving;
            enemy.dive_shot_timer = 0.5;
        }
    }

    if capture_attack {
        let points = capture_approach_path(transform, player_x);
        set_path(
            world,
            chosen,
            points,
            1.4 / resources.difficulty.dive_speed_multiplier.max(0.6),
            PathKind::CaptureApproach,
        );
        let _ = world.insert_one(
            chosen,
            TractorBeamState {
                phase: TractorBeamPhase::Telegraph,
                timer: 0.7,
            },
        );
    } else {
        let variant = resources.rng.rng.gen_range(0..8);
        let points = dive_path(transform, slot_target, player_x, variant);
        set_path(
            world,
            chosen,
            points,
            2.8 / resources.difficulty.dive_speed_multiplier.max(0.6),
            PathKind::Dive,
        );
    }

    resources.difficulty.dive_timer =
        resources.difficulty.dive_interval * resources.rng.rng.gen_range(0.82_f32..1.18_f32);
}

pub(super) fn update_enemy_paths(world: &mut World, resources: &mut Resources, dt: f32) {
    let mut to_remove_path: Vec<Entity> = Vec::new();
    let mut to_remove_timer: Vec<Entity> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();
    let mut to_return: Vec<Entity> = Vec::new();

    for (entity, transform, path, timer, enemy, slot, renderable, collider, beam) in world
        .query_mut::<(
            Entity,
            &mut Transform,
            &mut PathFollower,
            &mut crate::components::Timer,
            &mut Enemy,
            Option<&crate::components::FormationSlot>,
            &mut RenderablePrimitive,
            &mut Collider,
            &mut TractorBeamState,
        )>()
    {
        if timer.remaining > 0.0 {
            timer.remaining -= dt;
            continue;
        }

        let speed_mul = if matches!(
            enemy.mode,
            EnemyMode::Diving | EnemyMode::Returning | EnemyMode::Capturing
        ) {
            resources.difficulty.dive_speed_multiplier
        } else {
            1.0
        };

        path.t += (dt / path.duration.max(0.01)) * speed_mul;
        transform.pos = sample_path(&path.points, path.t);
        path.finished = path.t >= 1.0;

        if enemy.mode == EnemyMode::Diving
            && enemy.kind == EnemyKind::Butterfly
            && enemy.can_morph
            && !enemy.morphed
            && path.t >= 0.45
        {
            enemy.kind = EnemyKind::GalaxianFlagship;
            enemy.morphed = true;
            let (radius, appearance) = enemy_appearance(enemy.kind);
            *renderable = appearance;
            collider.radius = radius;
        }

        if !path.finished {
            continue;
        }

        match path.kind {
            PathKind::Challenge => {
                to_despawn.push(entity);
            }
            PathKind::Entry => {
                if resources.stage.stage_type == StageType::Challenge {
                    to_despawn.push(entity);
                } else if let Some(slot) = slot {
                    enemy.mode = EnemyMode::Formed;
                    transform.pos = slot.target;
                    to_remove_path.push(entity);
                    to_remove_timer.push(entity);
                }
            }
            PathKind::Dive | PathKind::CaptureApproach | PathKind::Return => {
                if let Some(slot) = slot {
                    enemy.mode = EnemyMode::Formed;
                    transform.pos = slot.target;
                    to_remove_path.push(entity);
                    to_remove_timer.push(entity);
                    beam.phase = TractorBeamPhase::Idle;
                    beam.timer = 0.0;
                } else {
                    to_return.push(entity);
                }
            }
        }
    }

    for entity in to_return {
        let start = world.get::<&Transform>(entity).ok().map(|t| t.pos);
        let target = world
            .get::<&crate::components::FormationSlot>(entity)
            .ok()
            .map(|s| s.target);
        if let (Some(start), Some(target)) = (start, target) {
            set_path(
                world,
                entity,
                return_path(start, target),
                1.3,
                PathKind::Return,
            );
            if let Ok(mut enemy) = world.get::<&mut Enemy>(entity) {
                enemy.mode = EnemyMode::Returning;
            }
        }
    }

    for entity in to_remove_path {
        let _ = world.remove_one::<PathFollower>(entity);
    }
    for entity in to_remove_timer {
        let _ = world.remove_one::<crate::components::Timer>(entity);
    }
    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

pub(super) fn update_capture_beams(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.stage.stage_type != StageType::Normal {
        return;
    }

    let player_entity = find_player_entity(world);
    let player_pos =
        player_entity.and_then(|entity| world.get::<&Transform>(entity).ok().map(|t| t.pos));

    let mut to_return = Vec::new();
    let mut capture_events = Vec::new();

    for (entity, enemy, beam, transform) in
        world.query_mut::<(Entity, &mut Enemy, &mut TractorBeamState, &Transform)>()
    {
        if enemy.mode != EnemyMode::Capturing {
            continue;
        }

        beam.timer -= dt;

        match beam.phase {
            TractorBeamPhase::Telegraph => {
                if beam.timer <= 0.0 {
                    beam.phase = TractorBeamPhase::Active;
                    beam.timer = 1.0;
                }
            }
            TractorBeamPhase::Active => {
                if let Some(pos) = player_pos {
                    if !resources.player.dual_active
                        && !enemy.carrying_player
                        && (pos.x - transform.pos.x).abs() <= 22.0
                        && pos.y > transform.pos.y
                        && (pos.y - transform.pos.y) < 280.0
                    {
                        capture_events.push(GameEvent::PlayerCaptured { boss: entity });
                        beam.phase = TractorBeamPhase::Cooldown;
                        beam.timer = 0.25;
                        to_return.push(entity);
                    }
                }

                if beam.timer <= 0.0 {
                    beam.phase = TractorBeamPhase::Cooldown;
                    beam.timer = 0.25;
                    to_return.push(entity);
                }
            }
            TractorBeamPhase::Cooldown => {
                if beam.timer <= 0.0 {
                    beam.phase = TractorBeamPhase::Idle;
                    beam.timer = 0.0;
                }
            }
            TractorBeamPhase::Idle => {}
        }
    }

    for event in capture_events {
        resources.events.events.push(event);
    }

    for entity in to_return {
        let start = world.get::<&Transform>(entity).ok().map(|t| t.pos);
        let target = world
            .get::<&crate::components::FormationSlot>(entity)
            .ok()
            .map(|s| s.target);
        if let (Some(start), Some(target)) = (start, target) {
            set_path(
                world,
                entity,
                return_path(start, target),
                1.2,
                PathKind::Return,
            );
            if let Ok(mut enemy) = world.get::<&mut Enemy>(entity) {
                enemy.mode = EnemyMode::Returning;
            }
        }
    }
}

pub(super) fn enemy_fire(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode != GameMode::Playing || !can_enemy_fire(resources.stage.stage_type) {
        return;
    }

    resources.difficulty.enemy_fire_timer -= dt;
    if resources.difficulty.enemy_fire_timer > 0.0 {
        return;
    }

    let player_pos = player_position(world).unwrap_or(vec2(SCREEN_WIDTH * 0.5, PLAYER_Y));

    let candidates: Vec<Entity> = world
        .query::<(Entity, &Enemy)>()
        .iter()
        .filter_map(|(entity, enemy)| {
            if matches!(
                enemy.mode,
                EnemyMode::Formed | EnemyMode::Diving | EnemyMode::Capturing
            ) {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        resources.difficulty.enemy_fire_timer = resources.difficulty.enemy_fire_interval;
        return;
    }

    let shooter_entity = candidates[resources.rng.rng.gen_range(0..candidates.len())];
    if let Ok(shooter_pos) = world.get::<&Transform>(shooter_entity) {
        let mut dir = player_pos - shooter_pos.pos;
        if dir.length_squared() < 1.0 {
            dir = vec2(0.0, 1.0);
        }
        dir = dir.normalize();

        resources
            .events
            .events
            .push(GameEvent::SpawnEnemyProjectile {
                position: shooter_pos.pos + vec2(0.0, 14.0),
                velocity: dir * resources.difficulty.enemy_bullet_speed,
            });
    }

    resources.difficulty.enemy_fire_timer =
        resources.difficulty.enemy_fire_interval * resources.rng.rng.gen_range(0.72_f32..1.22_f32);
}
