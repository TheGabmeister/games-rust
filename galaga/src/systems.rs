use ::rand::Rng;
use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    CapturedShipState, Collider, DualFighterWingman, Enemy, EnemyKind, EnemyMode, PathFollower,
    PathKind, Player, Projectile, ProjectileOwner, RenderablePrimitive, Timer, TractorBeamPhase,
    TractorBeamState, Transform, Velocity,
};
use crate::constants::{
    PLAYER_FIRE_COOLDOWN, PLAYER_RADIUS_DUAL, PLAYER_RADIUS_SINGLE, PLAYER_SHOT_SPEED,
    PLAYER_SPEED, PLAYER_Y, RESPAWN_DELAY, RESPAWN_INVULN, SCREEN_HEIGHT, SCREEN_WIDTH,
    STAGE_CLEAR_DELAY,
};
use crate::events::GameEvent;
use crate::patterns::{
    capture_approach_path, challenge_path, dive_path, entry_path, formation_slots, return_path,
    sample_path, stage_enemy_kinds,
};
use crate::resources::{GameMode, Resources, SpawnCommand, StageType};
use crate::rules::{
    CapturedShipOutcome, can_enemy_fire, difficulty_for_stage, enemy_base_hp,
    resolve_captured_ship_outcome, score_for_enemy, stage_is_cleared, stage_type_for,
};

pub fn fixed_update(world: &mut World, resources: &mut Resources, dt: f32) {
    read_input(resources);

    if resources.input.start_pressed {
        if matches!(resources.flow.mode, GameMode::Attract | GameMode::GameOver) {
            start_new_game(world, resources);
        }
    }

    if resources.input.pause_pressed {
        toggle_pause(resources);
    }

    if resources.flow.mode == GameMode::Pause {
        return;
    }

    resources.flow.mode_timer += dt;
    resources.ui.message_timer = (resources.ui.message_timer - dt).max(0.0);

    match resources.flow.mode {
        GameMode::Attract => return,
        GameMode::GameOver => return,
        _ => {}
    }

    handle_player_respawn(world, resources, dt);

    player_motion_and_fire(world, resources, dt);
    schedule_enemy_dives(world, resources, dt);
    update_enemy_paths(world, resources, dt);
    update_capture_beams(world, resources, dt);
    enemy_fire(world, resources, dt);
    move_projectiles(world, dt);
    detect_collisions(world, resources);
    process_events(world, resources);
    cleanup_entities(world, resources);
    flush_spawn_queue(world, resources);
    update_progression(world, resources, dt);

    if resources.score.score > resources.hi_score.value {
        resources.hi_score.value = resources.score.score;
    }
}

fn read_input(resources: &mut Resources) {
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    resources.input.move_axis = match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    resources.input.fire_pressed = is_key_pressed(KeyCode::Space);
    resources.input.start_pressed = is_key_pressed(KeyCode::Enter);
    resources.input.pause_pressed = is_key_pressed(KeyCode::P);
}

fn toggle_pause(resources: &mut Resources) {
    if resources.flow.mode == GameMode::Pause {
        resources.flow.mode = resources.flow.mode_before_pause;
        resources.flow.mode_timer = 0.0;
    } else if !matches!(resources.flow.mode, GameMode::Attract | GameMode::GameOver) {
        resources.flow.mode_before_pause = resources.flow.mode;
        resources.flow.mode = GameMode::Pause;
        resources.flow.mode_timer = 0.0;
    }
}

fn start_new_game(world: &mut World, resources: &mut Resources) {
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

fn spawn_player(world: &mut World, resources: &mut Resources, with_invuln: bool) {
    if find_player_entity(world).is_some() {
        return;
    }

    let dual = resources.player.dual_active;
    let radius = if dual {
        PLAYER_RADIUS_DUAL
    } else {
        PLAYER_RADIUS_SINGLE
    };
    let mut renderable = RenderablePrimitive {
        shape: crate::components::PrimitiveShape::Triangle,
        size: vec2(22.0, 24.0),
        color: SKYBLUE,
        layer: 40,
    };

    if dual {
        renderable.color = BLUE;
    }

    let entity = world.spawn((
        Transform {
            pos: vec2(SCREEN_WIDTH * 0.5, PLAYER_Y),
        },
        Velocity { vel: Vec2::ZERO },
        Collider { radius },
        renderable,
        Player {
            fire_cooldown: 0.0,
            invuln_timer: if with_invuln {
                RESPAWN_INVULN
            } else {
                resources.player.invuln_on_spawn
            },
        },
    ));

    if dual {
        let _ = world.insert_one(entity, DualFighterWingman { offset: 18.0 });
    }
}

fn find_player_entity(world: &World) -> Option<Entity> {
    world
        .query::<(Entity, &Player)>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
}

fn player_motion_and_fire(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode != GameMode::Playing {
        return;
    }

    let mut busy_barrels = [false; 2];
    for projectile in world.query::<&Projectile>().iter() {
        if projectile.owner == ProjectileOwner::Player {
            let idx = projectile.barrel as usize;
            if idx < busy_barrels.len() {
                busy_barrels[idx] = true;
            }
        }
    }

    let dual_active = resources.player.dual_active;
    let available_barrels = available_player_barrels(dual_active, busy_barrels);

    let mut spawn_requests = Vec::new();
    for (transform, velocity, player) in
        world.query_mut::<(&mut Transform, &mut Velocity, &mut Player)>()
    {
        velocity.vel.x = resources.input.move_axis * PLAYER_SPEED;
        transform.pos.x = (transform.pos.x + velocity.vel.x * dt).clamp(24.0, SCREEN_WIDTH - 24.0);
        player.fire_cooldown = (player.fire_cooldown - dt).max(0.0);
        player.invuln_timer = (player.invuln_timer - dt).max(0.0);

        if resources.input.fire_pressed && player.fire_cooldown <= 0.0 {
            for barrel in &available_barrels {
                let x_offset = match barrel {
                    0 => {
                        if dual_active {
                            -18.0
                        } else {
                            0.0
                        }
                    }
                    _ => 18.0,
                };
                spawn_requests.push(GameEvent::SpawnPlayerProjectile {
                    position: vec2(transform.pos.x + x_offset, transform.pos.y - 18.0),
                    barrel: *barrel,
                });
            }

            if !spawn_requests.is_empty() {
                player.fire_cooldown = PLAYER_FIRE_COOLDOWN;
            }
        }
    }

    resources.events.events.extend(spawn_requests);
}

fn schedule_enemy_dives(world: &mut World, resources: &mut Resources, dt: f32) {
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
fn update_enemy_paths(world: &mut World, resources: &mut Resources, dt: f32) {
    let mut to_remove_path: Vec<Entity> = Vec::new();
    let mut to_remove_timer: Vec<Entity> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();
    let mut to_return: Vec<Entity> = Vec::new();

    for (entity, transform, path, timer, enemy, slot, renderable, collider, beam) in world
        .query_mut::<(
            Entity,
            &mut Transform,
            &mut PathFollower,
            &mut Timer,
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
        let _ = world.remove_one::<Timer>(entity);
    }
    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

fn update_capture_beams(world: &mut World, resources: &mut Resources, dt: f32) {
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

fn enemy_fire(world: &mut World, resources: &mut Resources, dt: f32) {
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

fn move_projectiles(world: &mut World, dt: f32) {
    let mut to_despawn = Vec::new();

    for (entity, transform, velocity, _projectile) in
        world.query_mut::<(Entity, &mut Transform, &Velocity, &Projectile)>()
    {
        transform.pos += velocity.vel * dt;
        if transform.pos.y < -40.0
            || transform.pos.y > SCREEN_HEIGHT + 40.0
            || transform.pos.x < -40.0
            || transform.pos.x > SCREEN_WIDTH + 40.0
        {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}
fn detect_collisions(world: &mut World, resources: &mut Resources) {
    let mut player_bullets = Vec::new();
    let mut enemy_bullets = Vec::new();

    for (entity, projectile, transform) in world.query::<(Entity, &Projectile, &Transform)>().iter()
    {
        match projectile.owner {
            ProjectileOwner::Player => {
                player_bullets.push((entity, transform.pos, projectile.damage));
            }
            ProjectileOwner::Enemy => {
                enemy_bullets.push((entity, transform.pos));
            }
        }
    }

    let mut enemies = Vec::new();
    for (entity, enemy, transform, collider) in world
        .query::<(Entity, &Enemy, &Transform, &Collider)>()
        .iter()
    {
        enemies.push((
            entity,
            enemy.kind,
            enemy.mode,
            enemy.carrying_player,
            transform.pos,
            collider.radius,
        ));
    }

    let mut bullet_hits = Vec::new();
    let mut bullets_to_remove = Vec::new();
    for (bullet_entity, bullet_pos, damage) in &player_bullets {
        for (enemy_entity, _, _, _, enemy_pos, enemy_radius) in &enemies {
            let distance = bullet_pos.distance(*enemy_pos);
            if distance <= *enemy_radius + 4.0 {
                bullet_hits.push((*enemy_entity, *damage));
                bullets_to_remove.push(*bullet_entity);
                break;
            }
        }
    }

    for bullet in bullets_to_remove {
        let _ = world.despawn(bullet);
    }

    let mut destroy_events = Vec::new();
    for (enemy_entity, damage) in bullet_hits {
        if let Ok(mut enemy) = world.get::<&mut Enemy>(enemy_entity) {
            enemy.hp -= damage;
            if enemy.hp <= 0 {
                destroy_events.push(GameEvent::EnemyDestroyed {
                    entity: enemy_entity,
                    kind: enemy.kind,
                    mode: enemy.mode,
                    carrying_player: enemy.carrying_player,
                });
            }
        }
    }

    resources.events.events.extend(destroy_events);

    if let Some(player_entity) = find_player_entity(world) {
        let player_pos = world.get::<&Transform>(player_entity).ok().map(|t| t.pos);
        let player_radius = world.get::<&Collider>(player_entity).ok().map(|c| c.radius);
        let invuln = world
            .get::<&Player>(player_entity)
            .ok()
            .map(|p| p.invuln_timer);

        if let (Some(player_pos), Some(player_radius), Some(invuln)) =
            (player_pos, player_radius, invuln)
        {
            if invuln <= 0.0 {
                let mut hit = false;

                for (bullet_entity, bullet_pos) in enemy_bullets {
                    if bullet_pos.distance(player_pos) <= player_radius + 4.0 {
                        let _ = world.despawn(bullet_entity);
                        hit = true;
                        break;
                    }
                }

                if !hit {
                    for (_, _, mode, _, enemy_pos, enemy_radius) in &enemies {
                        if matches!(
                            mode,
                            EnemyMode::Diving | EnemyMode::Capturing | EnemyMode::Returning
                        ) && enemy_pos.distance(player_pos) <= enemy_radius + player_radius
                        {
                            hit = true;
                            break;
                        }
                    }
                }

                if hit {
                    resources.events.events.push(GameEvent::PlayerHit);
                }
            }
        }
    }
}

fn process_events(world: &mut World, resources: &mut Resources) {
    let mut pending = std::mem::take(&mut resources.events.events);

    while !pending.is_empty() {
        let mut follow_up = Vec::new();

        for event in pending {
            match event {
                GameEvent::SpawnPlayerProjectile { position, barrel } => {
                    resources
                        .spawn_queue
                        .commands
                        .push(SpawnCommand::PlayerProjectile { position, barrel });
                }
                GameEvent::SpawnEnemyProjectile { position, velocity } => {
                    resources
                        .spawn_queue
                        .commands
                        .push(SpawnCommand::EnemyProjectile { position, velocity });
                }
                GameEvent::EnemyDestroyed {
                    entity,
                    kind,
                    mode,
                    carrying_player,
                    ..
                } => {
                    let _ = world.despawn(entity);
                    let was_diving = mode != EnemyMode::Formed;
                    let gained = score_for_enemy(kind, was_diving, resources.stage.stage_type);
                    resources.score.score = resources.score.score.saturating_add(gained);

                    if resources.stage.stage_type == StageType::Challenge {
                        resources.stage.challenge_hits += 1;
                    }

                    if let Some(outcome) = resolve_captured_ship_outcome(mode, carrying_player) {
                        match outcome {
                            CapturedShipOutcome::RescueDual => {
                                follow_up.push(GameEvent::RescueCapturedShip);
                            }
                            CapturedShipOutcome::BetrayLater => {
                                follow_up.push(GameEvent::QueueBetrayedCapturedShip);
                            }
                        }
                    }
                }
                GameEvent::PlayerHit => {
                    player_lost(world, resources);
                }
                GameEvent::PlayerCaptured { boss } => {
                    if let Ok(mut enemy) = world.get::<&mut Enemy>(boss) {
                        enemy.carrying_player = true;
                    }
                    let _ = world.insert_one(boss, CapturedShipState);
                    player_lost(world, resources);
                }
                GameEvent::StageCleared => {
                    if resources.flow.mode != GameMode::StageClear {
                        resources.flow.mode = GameMode::StageClear;
                        resources.flow.mode_timer = STAGE_CLEAR_DELAY;
                    }
                }
                GameEvent::RescueCapturedShip => {
                    resources.player.dual_active = true;
                    if let Some(player_entity) = find_player_entity(world) {
                        let _ =
                            world.insert_one(player_entity, DualFighterWingman { offset: 18.0 });
                        if let Ok(mut collider) = world.get::<&mut Collider>(player_entity) {
                            collider.radius = PLAYER_RADIUS_DUAL;
                        }
                        if let Ok(mut renderable) =
                            world.get::<&mut RenderablePrimitive>(player_entity)
                        {
                            renderable.color = BLUE;
                        }
                    }
                }
                GameEvent::QueueBetrayedCapturedShip => {
                    resources.stage.betrayed_queue =
                        resources.stage.betrayed_queue.saturating_add(1);
                }
            }
        }

        pending = follow_up;
    }
}

fn cleanup_entities(world: &mut World, resources: &mut Resources) {
    let mut to_despawn = Vec::new();

    for (entity, enemy, transform) in world.query::<(Entity, &Enemy, &Transform)>().iter() {
        if resources.stage.stage_type == StageType::Challenge
            && transform.pos.y > SCREEN_HEIGHT + 70.0
            && enemy.mode == EnemyMode::Entering
        {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

fn flush_spawn_queue(world: &mut World, resources: &mut Resources) {
    let commands = std::mem::take(&mut resources.spawn_queue.commands);

    for command in commands {
        match command {
            SpawnCommand::PlayerProjectile { position, barrel } => {
                world.spawn((
                    Transform { pos: position },
                    Velocity {
                        vel: vec2(0.0, -PLAYER_SHOT_SPEED),
                    },
                    Collider { radius: 4.0 },
                    RenderablePrimitive {
                        shape: crate::components::PrimitiveShape::Rect,
                        size: vec2(4.0, 14.0),
                        color: YELLOW,
                        layer: 20,
                    },
                    Projectile {
                        owner: ProjectileOwner::Player,
                        damage: 1,
                        barrel,
                    },
                ));
            }
            SpawnCommand::EnemyProjectile { position, velocity } => {
                world.spawn((
                    Transform { pos: position },
                    Velocity { vel: velocity },
                    Collider { radius: 4.0 },
                    RenderablePrimitive {
                        shape: crate::components::PrimitiveShape::Circle,
                        size: vec2(8.0, 8.0),
                        color: ORANGE,
                        layer: 20,
                    },
                    Projectile {
                        owner: ProjectileOwner::Enemy,
                        damage: 1,
                        barrel: 0,
                    },
                ));
            }
        }
    }
}

fn update_progression(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode == GameMode::Ready {
        if resources.flow.mode_timer >= 1.0 {
            resources.flow.mode = GameMode::Playing;
            resources.flow.mode_timer = 0.0;
        }
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

fn handle_player_respawn(world: &mut World, resources: &mut Resources, dt: f32) {
    if resources.flow.mode != GameMode::PlayerDeath {
        return;
    }

    resources.player.respawn_timer -= dt;
    if resources.player.respawn_timer <= 0.0 {
        if resources.lives.lives > 0 {
            resources.player.invuln_on_spawn = RESPAWN_INVULN;
            spawn_player(world, resources, true);
            resources.flow.mode = GameMode::Playing;
            resources.flow.mode_timer = 0.0;
        } else {
            resources.flow.mode = GameMode::GameOver;
            resources.flow.mode_timer = 0.0;
        }
    }
}

fn player_lost(world: &mut World, resources: &mut Resources) {
    if let Some(player_entity) = find_player_entity(world) {
        let _ = world.despawn(player_entity);
    }

    if resources.flow.mode == GameMode::PlayerDeath || resources.flow.mode == GameMode::GameOver {
        return;
    }

    resources.lives.lives -= 1;
    resources.player.dual_active = false;

    if resources.lives.lives <= 0 {
        resources.flow.mode = GameMode::GameOver;
        resources.flow.mode_timer = 0.0;
    } else {
        resources.player.respawn_timer = RESPAWN_DELAY;
        resources.flow.mode = GameMode::PlayerDeath;
        resources.flow.mode_timer = 0.0;
    }
}
fn set_path(world: &mut World, entity: Entity, points: Vec<Vec2>, duration: f32, kind: PathKind) {
    let _ = world.remove_one::<PathFollower>(entity);
    let _ = world.insert_one(
        entity,
        PathFollower {
            points,
            duration,
            t: 0.0,
            kind,
            finished: false,
        },
    );
    let _ = world.remove_one::<Timer>(entity);
    let _ = world.insert_one(entity, Timer { remaining: 0.0 });
}

fn player_position(world: &World) -> Option<Vec2> {
    world
        .query::<(&Player, &Transform)>()
        .iter()
        .next()
        .map(|(_, transform)| transform.pos)
}

fn enemy_appearance(kind: EnemyKind) -> (f32, RenderablePrimitive) {
    match kind {
        EnemyKind::Bee => (
            12.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Circle,
                size: vec2(22.0, 22.0),
                color: GOLD,
                layer: 30,
            },
        ),
        EnemyKind::Butterfly => (
            12.5,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(24.0, 24.0),
                color: PINK,
                layer: 30,
            },
        ),
        EnemyKind::BossGalaga => (
            15.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Rect,
                size: vec2(30.0, 22.0),
                color: RED,
                layer: 32,
            },
        ),
        EnemyKind::GalaxianFlagship => (
            13.5,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(28.0, 26.0),
                color: GREEN,
                layer: 31,
            },
        ),
        EnemyKind::CapturedFighter => (
            13.0,
            RenderablePrimitive {
                shape: crate::components::PrimitiveShape::Triangle,
                size: vec2(24.0, 24.0),
                color: ORANGE,
                layer: 31,
            },
        ),
    }
}

pub fn available_player_barrels(dual_active: bool, busy: [bool; 2]) -> Vec<u8> {
    let mut out = Vec::new();

    if !busy[0] {
        out.push(0);
    }

    if dual_active && !busy[1] {
        out.push(1);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::available_player_barrels;

    #[test]
    fn dual_fighter_uses_two_barrels_with_one_active_shot_each() {
        assert_eq!(available_player_barrels(false, [false, false]), vec![0]);
        assert_eq!(
            available_player_barrels(false, [true, false]),
            Vec::<u8>::new()
        );

        assert_eq!(available_player_barrels(true, [false, false]), vec![0, 1]);
        assert_eq!(available_player_barrels(true, [true, false]), vec![1]);
        assert_eq!(available_player_barrels(true, [false, true]), vec![0]);
        assert_eq!(
            available_player_barrels(true, [true, true]),
            Vec::<u8>::new()
        );
    }
}
