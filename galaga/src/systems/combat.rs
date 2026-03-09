use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    CapturedShipState, Collider, DualFighterWingman, Enemy, EnemyMode, Projectile, ProjectileOwner,
    RenderablePrimitive, Transform, Velocity,
};
use crate::constants::{
    PLAYER_RADIUS_DUAL, PLAYER_SHOT_SPEED, SCREEN_HEIGHT, SCREEN_WIDTH, STAGE_CLEAR_DELAY,
};
use crate::events::GameEvent;
use crate::resources::{GameMode, Resources, SpawnCommand, StageType};
use crate::rules::{CapturedShipOutcome, resolve_captured_ship_outcome, score_for_enemy};

use super::player::{find_player_entity, player_lost};

pub(super) fn move_projectiles(world: &mut World, dt: f32) {
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

pub(super) fn detect_collisions(world: &mut World, resources: &mut Resources) {
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
            .get::<&crate::components::Player>(player_entity)
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

pub(super) fn process_events(world: &mut World, resources: &mut Resources) {
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

pub(super) fn flush_spawn_queue(world: &mut World, resources: &mut Resources) {
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
