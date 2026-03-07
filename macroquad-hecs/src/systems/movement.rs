use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::ecs::{Bouncer, Collider, PreviousTransform, Transform, Velocity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBounds {
    min: Vec2,
    max: Vec2,
}

impl WorldBounds {
    pub fn from_size(size: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max: vec2(size.x.max(1.0), size.y.max(1.0)),
        }
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }
}

pub fn snapshot_previous_transforms(world: &mut World) {
    for (transform, previous) in world.query_mut::<(&Transform, &mut PreviousTransform)>() {
        previous.position = transform.position;
    }
}

pub fn integrate(world: &mut World, dt: f32) {
    for (transform, velocity) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.position += velocity.value * dt;
    }
}

pub fn bounce(world: &mut World, bounds: WorldBounds) {
    for (transform, velocity, collider, _) in
        world.query_mut::<(&mut Transform, &mut Velocity, &Collider, &Bouncer)>()
    {
        if transform.position.x <= bounds.min.x {
            transform.position.x = bounds.min.x;
            velocity.value.x = velocity.value.x.abs();
        } else if transform.position.x + collider.size.x >= bounds.max.x {
            transform.position.x = (bounds.max.x - collider.size.x).max(bounds.min.x);
            velocity.value.x = -velocity.value.x.abs();
        }

        if transform.position.y <= bounds.min.y {
            transform.position.y = bounds.min.y;
            velocity.value.y = velocity.value.y.abs();
        } else if transform.position.y + collider.size.y >= bounds.max.y {
            transform.position.y = (bounds.max.y - collider.size.y).max(bounds.min.y);
            velocity.value.y = -velocity.value.y.abs();
        }
    }
}

pub fn clamp_player(world: &mut World, player: Entity, bounds: WorldBounds) {
    let mut player_query = world.query_one::<(&mut Transform, &Collider)>(player);
    if let Ok((transform, collider)) = player_query.get() {
        transform.position.x = transform.position.x.clamp(
            bounds.min.x,
            (bounds.max.x - collider.size.x).max(bounds.min.x),
        );
        transform.position.y = transform.position.y.clamp(
            bounds.min.y,
            (bounds.max.y - collider.size.y).max(bounds.min.y),
        );
    }
}

#[cfg(test)]
mod tests {
    use hecs::World;

    use super::{WorldBounds, bounce, integrate, snapshot_previous_transforms};
    use crate::ecs::{Bouncer, Collider, PreviousTransform, Transform, Velocity};
    use macroquad::prelude::*;

    #[test]
    fn integrates_velocity_with_fixed_dt() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform {
                position: vec2(1.0, 2.0),
            },
            PreviousTransform {
                position: vec2(1.0, 2.0),
            },
            Velocity {
                value: vec2(3.0, -1.0),
            },
        ));

        snapshot_previous_transforms(&mut world);
        integrate(&mut world, 0.5);

        let transform = world
            .get::<&Transform>(entity)
            .expect("transform should exist");
        let previous = world
            .get::<&PreviousTransform>(entity)
            .expect("previous transform should exist");

        assert!((transform.position.x - 2.5).abs() < 0.0001);
        assert!((transform.position.y - 1.5).abs() < 0.0001);
        assert!((previous.position.x - 1.0).abs() < 0.0001);
        assert!((previous.position.y - 2.0).abs() < 0.0001);
    }

    #[test]
    fn bouncer_reflects_against_world_edges() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform {
                position: vec2(-2.0, 10.0),
            },
            Velocity {
                value: vec2(-40.0, 0.0),
            },
            Collider {
                size: vec2(16.0, 16.0),
            },
            Bouncer,
        ));

        bounce(&mut world, WorldBounds::from_size(vec2(100.0, 60.0)));

        let transform = world
            .get::<&Transform>(entity)
            .expect("transform should exist");
        let velocity = world
            .get::<&Velocity>(entity)
            .expect("velocity should exist");
        assert_eq!(transform.position.x, 0.0);
        assert!(velocity.value.x > 0.0);
    }
}
