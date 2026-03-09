use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    Collider, DualFighterWingman, Player, Projectile, ProjectileOwner, RenderablePrimitive,
    Transform, Velocity,
};
use crate::constants::{
    PLAYER_FIRE_COOLDOWN, PLAYER_RADIUS_DUAL, PLAYER_RADIUS_SINGLE, PLAYER_SPEED, PLAYER_Y,
    RESPAWN_DELAY, RESPAWN_INVULN, SCREEN_WIDTH,
};
use crate::events::GameEvent;
use crate::resources::{GameMode, Resources};

pub(super) fn spawn_player(world: &mut World, resources: &mut Resources, with_invuln: bool) {
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

pub(super) fn find_player_entity(world: &World) -> Option<Entity> {
    world
        .query::<(Entity, &Player)>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
}

pub(super) fn player_motion_and_fire(world: &mut World, resources: &mut Resources, dt: f32) {
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

pub(super) fn handle_player_respawn(world: &mut World, resources: &mut Resources, dt: f32) {
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

pub(super) fn player_lost(world: &mut World, resources: &mut Resources) {
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
