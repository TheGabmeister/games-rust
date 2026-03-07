use std::sync::atomic::{AtomicBool, Ordering};

use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::ecs::Name;

static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn toggle_debug() {
    DEBUG.store(!DEBUG.load(Ordering::Relaxed), Ordering::Relaxed);
}

pub fn debug_enabled() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

pub fn draw_debug_overlay(world: &World, state_name: &str) {
    let mut entity_lines = Vec::new();
    let mut query = world.query::<(Entity, &Name)>();
    for (entity, name) in query.iter() {
        entity_lines.push(format!("{entity:?} | {}", name.0));
    }
    entity_lines.sort();

    let mut lines = Vec::new();
    lines.push("DEBUG MENU".to_owned());
    lines.push(format!("State: {state_name}"));
    lines.push(format!("Entity count: {}", world.len()));
    lines.extend(entity_lines);

    let panel_width = 360.0;
    let line_height = 20.0;
    let panel_height = 18.0 + line_height * lines.len() as f32 + 10.0;
    let float_offset = (get_time() as f32 * 2.2).sin() * 4.0;
    let panel_x = screen_width() - panel_width - 16.0;
    let panel_y = 18.0 + float_offset;

    draw_rectangle(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        Color::new(0.06, 0.08, 0.1, 0.86),
    );
    draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 1.0, GREEN);

    for (i, line) in lines.iter().enumerate() {
        draw_text(
            line,
            panel_x + 10.0,
            panel_y + 24.0 + i as f32 * line_height,
            20.0,
            WHITE,
        );
    }
}
