use macroquad::prelude::*;
use crate::entities::Entity;

// AABB collision check between two entities.
pub fn aabb(a: &Entity, b: &Entity) -> bool {
    a.rect().overlaps(&b.rect())
}

// Returns all colliding (id_a, id_b) pairs from the entity list.
pub fn check_collisions(entities: &[Entity]) -> Vec<(u32, u32)> {
    let mut pairs = Vec::new();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if entities[i].active && entities[j].active && aabb(&entities[i], &entities[j]) {
                pairs.push((entities[i].id, entities[j].id));
            }
        }
    }
    pairs
}

// Apply velocity to position.
pub fn update_movement(entity: &mut Entity, dt: f32) {
    entity.position += entity.velocity * dt;
}

// Keep entity fully within screen bounds.
pub fn clamp_to_screen(entity: &mut Entity) {
    entity.position.x = entity.position.x.clamp(0.0, screen_width() - entity.size.x);
    entity.position.y = entity.position.y.clamp(0.0, screen_height() - entity.size.y);
}
