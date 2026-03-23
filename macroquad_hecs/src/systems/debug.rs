use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    ActivePowerups, BoxCollider, CircleCollider, Enemy, Pickup, Player, PowerupPickup, Projectile,
    ProjectileOwner, Transform,
};

/// Draw collider wireframes for all entities that have a collider component.
pub fn system_draw_colliders(world: &World) {
    // Box colliders (green)
    for (transform, col) in world.query::<(&Transform, &BoxCollider)>().iter() {
        let x = transform.pos.x - col.half.x;
        let y = transform.pos.y - col.half.y;
        let w = col.half.x * 2.0;
        let h = col.half.y * 2.0;
        draw_rectangle_lines(x, y, w, h, 1.5, GREEN);
    }

    // Circle colliders (yellow)
    for (transform, col) in world.query::<(&Transform, &CircleCollider)>().iter() {
        draw_circle_lines(transform.pos.x, transform.pos.y, col.radius, 1.5, YELLOW);
    }
}

// probes an entity's components one-by-one to figure out what "type" of game object it is,
// since there's no single name/type field on entities in an ECS.
fn entity_type_label(world: &World, entity: Entity) -> String {
    if world.get::<&Player>(entity).is_ok() {
        return "Player".into();
    }
    if let Ok(enemy) = world.get::<&Enemy>(entity) {
        return format!("Enemy ({:?})", enemy.kind);
    }
    if let Ok(proj) = world.get::<&Projectile>(entity) {
        return match proj.owner {
            ProjectileOwner::Player => "Bullet (Player)".into(),
            ProjectileOwner::Enemy => "Bullet (Enemy)".into(),
        };
    }
    if let Ok(pickup) = world.get::<&Pickup>(entity) {
        return format!("Pickup ({:?})", pickup.kind);
    }
    if let Ok(powerup) = world.get::<&PowerupPickup>(entity) {
        return format!("Powerup ({:?})", powerup.effect);
    }
    if let Ok(powerups) = world.get::<&ActivePowerups>(entity)
        && (powerups.bolt_remaining > 0.0 || powerups.shield_remaining > 0.0)
    {
        return format!(
            "Player Buffs (bolt {:.1}s, shield {:.1}s)",
            powerups.bolt_remaining, powerups.shield_remaining
        );
    }
    "Obstacle".into()
}

#[cfg(debug_assertions)]
pub fn system_draw_debug_ui(world: &World) {
    egui_macroquad::ui(|egui_ctx| {
        egui_macroquad::egui::Window::new("Entities")
            .default_pos([10.0, 80.0])
            .default_size([220.0, 300.0])
            .resizable(true)
            .show(egui_ctx, |ui| {
                ui.label(format!("Total: {}", world.len()));
                ui.separator();

                let mut entries: Vec<(Entity, String)> = world
                    .iter()
                    .map(|entity_ref| {
                        let e = entity_ref.entity();
                        let label = entity_type_label(world, e);
                        (e, label)
                    })
                    .collect();
                entries.sort_by_key(|(e, _)| e.id());

                egui_macroquad::egui::ScrollArea::vertical().show(ui, |ui| {
                    for (entity, label) in &entries {
                        ui.label(format!("[{}] {}", entity.id(), label));
                    }
                });
            });
    });
}
