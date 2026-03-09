use crate::constants::WORLD_WIDTH;
use macroquad::prelude::Rect;

/// Basic AABB overlap test.
pub fn aabb_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/// Wrap-aware AABB test: also tests `a` shifted by ±WORLD_WIDTH.
pub fn aabb_overlap_wrapped(a: Rect, b: Rect) -> bool {
    if aabb_overlap(a, b) {
        return true;
    }
    let a_right = Rect::new(a.x + WORLD_WIDTH, a.y, a.w, a.h);
    if aabb_overlap(a_right, b) {
        return true;
    }
    let a_left = Rect::new(a.x - WORLD_WIDTH, a.y, a.w, a.h);
    aabb_overlap(a_left, b)
}
