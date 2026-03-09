use hecs::World;
use macroquad::prelude::*;

use crate::components::{
    Collider, Enemy, EnemyKind, EnemyMode, PathFollower, PathKind, Timer, TractorBeamPhase,
    TractorBeamState, Transform, Velocity,
};
use crate::events::GameEvent;
use crate::patterns::{challenge_path, entry_path, formation_slots, stage_enemy_kinds};
use crate::resources::{GameMode, Resources, StageType};
use crate::rules::{difficulty_for_stage, enemy_base_hp, stage_is_cleared, stage_type_for};

use super::helpers::enemy_appearance;
use super::player::spawn_player;

pub(super) fn start_new_game(world: &mut World, resources: &mut Resources) {
    let hi_score = resources.hi_score.value;
    *resources = Resources::default();
    resources.hi_score.value = hi_score;
    resources.flow.mode = GameMode::Ready;
    resources.flow.mode_timer = 0.0;
    resources.stage.number = 1;
    resources.lives.lives = 3;
    resources.player.dual_active = false;

    start_stage(world, resources, true);
}

fn start_stage(world: &mut World, resources: &mut Resources, spawn_player_now: bool) {
    world.clear();

    resources.stage.stage_type = stage_type_for(resources.stage.number);
    resources.stage.spawn_finished = false;
    resources.stage.challenge_hits = 0;
    resources.stage.challenge_total = 0;

    let tuning = difficulty_for_stage(resources.stage.number);
    resources.difficulty.dive_interval = tuning.dive_interval;
    resources.difficulty.dive_timer = tuning.dive_interval * 0.7;
    resources.difficulty.max_divers = tuning.max_divers;
    resources.difficulty.enemy_fire_interval = tuning.enemy_fire_interval;
    resources.difficulty.enemy_fire_timer = tuning.enemy_fire_interval * 0.6;
    resources.difficulty.enemy_bullet_speed = tuning.enemy_bullet_speed;
    resources.difficulty.dive_speed_multiplier = tuning.dive_speed_multiplier;

    match resources.stage.stage_type {
        StageType::Normal => spawn_normal_stage(world, resources),
        StageType::Challenge => spawn_challenge_stage(world, resources),
    }

    resources.stage.spawn_finished = true;

    if spawn_player_now {
        spawn_player(world, resources, true);
    }

    resources.flow.mode = GameMode::Ready;
    resources.flow.mode_timer = 0.0;
    resources.ui.message_timer = 1.2;
}

fn spawn_normal_stage(world: &mut World, resources: &mut Resources) {
    let slots = formation_slots();
    let enemy_kinds = stage_enemy_kinds(resources.stage.betrayed_queue);
    resources.stage.betrayed_queue = resources.stage.betrayed_queue.saturating_sub(
        enemy_kinds
            .iter()
            .filter(|kind| **kind == EnemyKind::CapturedFighter)
            .count() as u32,
    );

    for (index, (slot, kind)) in slots.into_iter().zip(enemy_kinds.into_iter()).enumerate() {
        let points = entry_path(slot, index);
        let start = points[0];
        let (radius, renderable) = enemy_appearance(kind);
        let can_morph = kind == EnemyKind::Butterfly && index % 3 == 0;

        world.spawn((
            Transform { pos: start },
            Velocity { vel: Vec2::ZERO },
            Collider { radius },
            renderable,
            Enemy {
                kind,
                mode: EnemyMode::Entering,
                hp: enemy_base_hp(kind),
                can_morph,
                morphed: false,
                carrying_player: false,
                dive_shot_timer: 0.8 + (index % 5) as f32 * 0.18,
            },
            crate::components::FormationSlot { target: slot },
            PathFollower {
                points,
                duration: 1.8 + index as f32 * 0.045,
                t: 0.0,
                kind: PathKind::Entry,
                finished: false,
            },
            Timer {
                remaining: index as f32 * 0.06,
            },
            TractorBeamState {
                phase: TractorBeamPhase::Idle,
                timer: 0.0,
            },
        ));
    }
}

fn spawn_challenge_stage(world: &mut World, resources: &mut Resources) {
    let total = 40;
    resources.stage.challenge_total = total;

    for index in 0..total {
        let kind = match index % 10 {
            0 => EnemyKind::BossGalaga,
            1 | 2 | 3 | 4 => EnemyKind::Butterfly,
            _ => EnemyKind::Bee,
        };

        let points = challenge_path(index as usize);
        let start = points[0];
        let (radius, mut renderable) = enemy_appearance(kind);
        renderable.color = Color::new(
            (renderable.color.r + 0.2).min(1.0),
            (renderable.color.g + 0.2).min(1.0),
            (renderable.color.b + 0.2).min(1.0),
            1.0,
        );

        world.spawn((
            Transform { pos: start },
            Velocity { vel: Vec2::ZERO },
            Collider { radius },
            renderable,
            Enemy {
                kind,
                mode: EnemyMode::Entering,
                hp: enemy_base_hp(kind),
                can_morph: false,
                morphed: false,
                carrying_player: false,
                dive_shot_timer: 999.0,
            },
            PathFollower {
                points,
                duration: 3.0 + (index / 8) as f32 * 0.2,
                t: 0.0,
                kind: PathKind::Challenge,
                finished: false,
            },
            Timer {
                remaining: index as f32 * 0.12,
            },
            TractorBeamState {
                phase: TractorBeamPhase::Idle,
                timer: 0.0,
            },
        ));
    }
}

pub(super) fn cleanup_entities(world: &mut World, resources: &mut Resources) {
    let mut to_despawn = Vec::new();

    for (entity, enemy, transform) in world.query::<(hecs::Entity, &Enemy, &Transform)>().iter() {
        if resources.stage.stage_type == StageType::Challenge
            && transform.pos.y > crate::constants::SCREEN_HEIGHT + 70.0
            && enemy.mode == EnemyMode::Entering
        {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

pub(super) fn update_progression(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode == GameMode::Ready && resources.flow.mode_timer >= 1.0 {
        resources.flow.mode = GameMode::Playing;
        resources.flow.mode_timer = 0.0;
    }

    let enemies_remaining = world.query::<&Enemy>().iter().count();
    if matches!(resources.flow.mode, GameMode::Playing | GameMode::Ready)
        && stage_is_cleared(resources.stage.spawn_finished, enemies_remaining)
    {
        resources.events.events.push(GameEvent::StageCleared);
    }

    if resources.flow.mode == GameMode::StageClear {
        resources.flow.mode_timer -= dt;
        if resources.flow.mode_timer <= 0.0 {
            if resources.stage.stage_type == StageType::Challenge
                && resources.stage.challenge_total > 0
                && resources.stage.challenge_hits >= resources.stage.challenge_total
            {
                resources.score.score = resources.score.score.saturating_add(10_000);
            }

            resources.stage.number += 1;
            start_stage(world, resources, true);
        }
    }
}
