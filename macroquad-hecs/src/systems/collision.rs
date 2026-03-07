use hecs::{Entity, World};
use macroquad::prelude::Rect;

use crate::ecs::{Collider, CollisionState, Name, Player, Transform};

/// Detects AABB collisions between the player and all other collidable entities.
/// Results are written directly into the player's `CollisionState` component,
/// so callers do not need to pass entity handles or track previous state.
pub fn detect_player_collisions(world: &mut World) {
    // Step 1: find the player entity, its rect, and the previous collision state.
    let player_data = {
        let mut q = world.query::<(Entity, &Player, &Transform, &Collider, &CollisionState)>();
        q.iter().next().map(|(entity, _, t, c, state)| {
            let rect = Rect::new(t.position.x, t.position.y, c.size.x, c.size.y);
            (entity, rect, state.is_colliding)
        })
    };

    let Some((player_entity, player_rect, was_colliding)) = player_data else {
        return;
    };

    // Step 2: collect collision notes against every non-player collidable.
    let notes = {
        let mut q = world.query::<(Entity, &Name, &Transform, &Collider)>();
        let mut notes = Vec::new();
        for (entity, name, t, c) in q.iter() {
            if entity == player_entity {
                continue;
            }
            let other = Rect::new(t.position.x, t.position.y, c.size.x, c.size.y);
            if player_rect.overlaps(&other) {
                notes.push(format!("Player collides with {}", name.0));
            }
        }
        notes
    };

    // Step 3: write results back into the player's CollisionState component.
    if let Ok(mut state) = world.get::<&mut CollisionState>(player_entity) {
        let is_colliding = !notes.is_empty();
        state.started_colliding = is_colliding && !was_colliding;
        state.is_colliding = is_colliding;
        state.notes = notes;
    }
}

#[cfg(test)]
mod tests {
    use hecs::World;
    use macroquad::prelude::*;

    use super::detect_player_collisions;
    use crate::ecs::{Collider, CollisionState, Name, Player, Transform};

    fn spawn_player(world: &mut World, pos: Vec2) -> hecs::Entity {
        world.spawn((
            Name("Player".to_owned()),
            Transform { position: pos },
            Collider {
                size: vec2(10.0, 10.0),
            },
            Player,
            CollisionState::default(),
        ))
    }

    #[test]
    fn detects_overlap_and_collision_enter() {
        let mut world = World::new();
        let player = spawn_player(&mut world, vec2(0.0, 0.0));
        world.spawn((
            Name("Wall".to_owned()),
            Transform {
                position: vec2(5.0, 0.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));

        detect_player_collisions(&mut world);

        let state = world
            .get::<&CollisionState>(player)
            .expect("CollisionState should exist");
        assert!(state.is_colliding);
        assert!(state.started_colliding);
        assert_eq!(state.notes.len(), 1);
    }

    #[test]
    fn reports_no_collision_when_apart() {
        let mut world = World::new();
        let player = spawn_player(&mut world, vec2(0.0, 0.0));
        world.spawn((
            Name("Wall".to_owned()),
            Transform {
                position: vec2(30.0, 30.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));

        detect_player_collisions(&mut world);

        let state = world
            .get::<&CollisionState>(player)
            .expect("CollisionState should exist");
        assert!(!state.is_colliding);
        assert!(!state.started_colliding);
        assert!(state.notes.is_empty());
    }
}
