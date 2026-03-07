use macroquad::prelude::*;
use crate::entities::Entity;

// Returns all colliding (id_a, id_b) pairs from the entity list.
pub fn check_collisions(entities: &[Entity]) -> Vec<(u32, u32)> {
    let mut pairs = Vec::new();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if entities[i].active && entities[j].active
                && entities[i].rect().overlaps(&entities[j].rect())
            {
                pairs.push((entities[i].id, entities[j].id));
            }
        }
    }
    pairs
}

// Keep entity fully within screen bounds.
pub fn clamp_to_screen(entity: &mut Entity) {
    entity.position.x = entity.position.x.clamp(0.0, screen_width() - entity.size.x);
    entity.position.y = entity.position.y.clamp(0.0, screen_height() - entity.size.y);
}
