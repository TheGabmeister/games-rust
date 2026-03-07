use hecs::{Entity, World};
use macroquad::prelude::Rect;

use crate::ecs::{Collider, Name, Transform};

#[derive(Debug, Default)]
pub struct CollisionReport {
    pub notes: Vec<String>,
    pub is_colliding: bool,
    pub started_colliding: bool,
}

pub fn detect_player_collisions(
    world: &World,
    player: Entity,
    was_colliding: bool,
) -> CollisionReport {
    let Some(player_rect) = entity_rect(world, player) else {
        return CollisionReport::default();
    };

    let mut notes = Vec::new();
    let mut query = world.query::<(Entity, &Name, &Transform, &Collider)>();
    for (entity, name, transform, collider) in query.iter() {
        if entity == player {
            continue;
        }

        let other = Rect::new(
            transform.position.x,
            transform.position.y,
            collider.size.x,
            collider.size.y,
        );

        if player_rect.overlaps(&other) {
            notes.push(format!("Player collides with {}", name.0));
        }
    }

    let is_colliding = !notes.is_empty();
    CollisionReport {
        notes,
        is_colliding,
        started_colliding: is_colliding && !was_colliding,
    }
}

fn entity_rect(world: &World, entity: Entity) -> Option<Rect> {
    let transform = world.get::<&Transform>(entity).ok()?;
    let collider = world.get::<&Collider>(entity).ok()?;
    Some(Rect::new(
        transform.position.x,
        transform.position.y,
        collider.size.x,
        collider.size.y,
    ))
}

#[cfg(test)]
mod tests {
    use hecs::World;
    use macroquad::prelude::*;

    use super::detect_player_collisions;
    use crate::ecs::{Collider, Name, Transform};

    #[test]
    fn detects_overlap_and_collision_enter() {
        let mut world = World::new();
        let player = world.spawn((
            Name("Player".to_owned()),
            Transform {
                position: vec2(0.0, 0.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));
        world.spawn((
            Name("Wall".to_owned()),
            Transform {
                position: vec2(5.0, 0.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));

        let report = detect_player_collisions(&world, player, false);
        assert!(report.is_colliding);
        assert!(report.started_colliding);
        assert_eq!(report.notes.len(), 1);
    }

    #[test]
    fn reports_no_collision_when_apart() {
        let mut world = World::new();
        let player = world.spawn((
            Name("Player".to_owned()),
            Transform {
                position: vec2(0.0, 0.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));
        world.spawn((
            Name("Wall".to_owned()),
            Transform {
                position: vec2(30.0, 30.0),
            },
            Collider {
                size: vec2(10.0, 10.0),
            },
        ));

        let report = detect_player_collisions(&world, player, false);
        assert!(!report.is_colliding);
        assert!(!report.started_colliding);
        assert!(report.notes.is_empty());
    }
}
