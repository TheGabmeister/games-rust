use std::sync::atomic::{AtomicBool, Ordering};
use macroquad::prelude::*;
use crate::entities::Entity;

static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn is_debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

pub fn toggle_debug() {
    DEBUG.fetch_xor(true, Ordering::Relaxed);
}

pub fn draw_debug_ui(entities: &[Entity]) {
    if !is_debug() {
        return;
    }

    const PANEL_X: f32 = 10.0;
    const PANEL_Y: f32 = 10.0;
    const PANEL_W: f32 = 260.0;
    const ROW_H: f32 = 18.0;
    const PAD: f32 = 8.0;
    const FONT_SIZE: f32 = 14.0;

    let rows = 1.0 + entities.len() as f32; // header + one row per entity
    let panel_h = PAD + ROW_H * rows + PAD;

    // Background + border
    draw_rectangle(PANEL_X, PANEL_Y, PANEL_W, panel_h, Color::new(0.0, 0.0, 0.0, 0.78));
    draw_rectangle_lines(PANEL_X, PANEL_Y, PANEL_W, panel_h, 1.0, DARKGRAY);

    // Header
    draw_text(
        &format!("[ DEBUG ]  entities: {}", entities.len()),
        PANEL_X + PAD,
        PANEL_Y + PAD + FONT_SIZE,
        FONT_SIZE,
        YELLOW,
    );

    // One row per entity
    for (i, e) in entities.iter().enumerate() {
        let y = PANEL_Y + PAD + ROW_H * (i as f32 + 2.0);
        let active_str = if e.active { "" } else { " [inactive]" };
        let label = format!(
            "#{} {}  pos({:.0},{:.0})  sz({:.0},{:.0}){}",
            e.id, e.name, e.position.x, e.position.y, e.size.x, e.size.y, active_str
        );
        draw_text(&label, PANEL_X + PAD, y, FONT_SIZE, WHITE);
    }
}
