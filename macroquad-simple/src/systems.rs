use macroquad::prelude::*;
use crate::entities::{Entity, EntityKind};

const PLAYER_SPEED: f32 = 220.0;

pub fn apply_player_input(entities: &mut [Entity]) {
    let Some(player) = entities.iter_mut().find(|e| e.kind == EntityKind::Player) else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { dir.y -= 1.0; }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)   { dir.y += 1.0; }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)   { dir.x -= 1.0; }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)  { dir.x += 1.0; }

    player.velocity = if dir != Vec2::ZERO {
        dir.normalize() * PLAYER_SPEED
    } else {
        Vec2::ZERO
    };
}

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
