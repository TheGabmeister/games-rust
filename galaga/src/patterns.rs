use macroquad::prelude::*;

use crate::components::EnemyKind;
use crate::constants::{
    FORMATION_COL_GAP, FORMATION_ROW_GAP, FORMATION_TOP, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub fn formation_slots() -> Vec<Vec2> {
    let center_x = SCREEN_WIDTH * 0.5;
    let mut slots = Vec::with_capacity(40);

    for col in [2.0, 3.0, 4.0, 5.0] {
        slots.push(vec2(
            center_x + (col - 3.5) * FORMATION_COL_GAP,
            FORMATION_TOP,
        ));
    }

    for col in 0..8 {
        slots.push(vec2(
            center_x + (col as f32 - 3.5) * FORMATION_COL_GAP,
            FORMATION_TOP + FORMATION_ROW_GAP,
        ));
    }

    for col in [2.0, 3.0, 4.0, 5.0] {
        slots.push(vec2(
            center_x + (col - 3.5) * FORMATION_COL_GAP,
            FORMATION_TOP + FORMATION_ROW_GAP * 2.0,
        ));
    }

    for row in 3..6 {
        for col in 0..8 {
            slots.push(vec2(
                center_x + (col as f32 - 3.5) * FORMATION_COL_GAP,
                FORMATION_TOP + FORMATION_ROW_GAP * row as f32,
            ));
        }
    }

    slots
}

pub fn stage_enemy_kinds(betrayed_queue: u32) -> Vec<EnemyKind> {
    let mut kinds = Vec::with_capacity(40);
    kinds.extend(std::iter::repeat_n(EnemyKind::BossGalaga, 4));
    kinds.extend(std::iter::repeat_n(EnemyKind::Butterfly, 12));
    kinds.extend(std::iter::repeat_n(EnemyKind::Bee, 24));

    let replace = betrayed_queue.min(24) as usize;
    for i in 0..replace {
        kinds[16 + i] = EnemyKind::CapturedFighter;
    }
    kinds
}

pub fn entry_path(slot: Vec2, index: usize) -> Vec<Vec2> {
    let side = if index % 2 == 0 { -1.0 } else { 1.0 };
    let start_x = if side < 0.0 {
        -60.0 - (index as f32 % 3.0) * 20.0
    } else {
        SCREEN_WIDTH + 60.0 + (index as f32 % 3.0) * 20.0
    };
    let start_y = -60.0 - ((index % 5) as f32) * 24.0;
    let bend = 80.0 + ((index % 7) as f32) * 12.0;

    vec![
        vec2(start_x, start_y),
        vec2(
            SCREEN_WIDTH * 0.5 + side * bend,
            80.0 + (index % 4) as f32 * 20.0,
        ),
        vec2(slot.x + side * 40.0, slot.y - 80.0),
        slot,
    ]
}

pub fn dive_path(start: Vec2, slot: Vec2, player_x: f32, variant: usize) -> Vec<Vec2> {
    let dir = if variant.is_multiple_of(2) { -1.0 } else { 1.0 };
    let target_x = player_x.clamp(80.0, SCREEN_WIDTH - 80.0) + dir * (variant % 3) as f32 * 26.0;
    let side_x = (target_x + dir * 110.0).clamp(40.0, SCREEN_WIDTH - 40.0);

    vec![
        start,
        vec2(start.x + dir * 45.0, start.y + 90.0),
        vec2(target_x, SCREEN_HEIGHT * 0.58),
        vec2(side_x, SCREEN_HEIGHT - 40.0),
        vec2(slot.x, slot.y),
    ]
}

pub fn capture_approach_path(start: Vec2, player_x: f32) -> Vec<Vec2> {
    let target_x = player_x.clamp(90.0, SCREEN_WIDTH - 90.0);
    vec![
        start,
        vec2(start.x, start.y + 70.0),
        vec2(target_x, 220.0),
        vec2(target_x, 220.0),
    ]
}

pub fn return_path(start: Vec2, slot: Vec2) -> Vec<Vec2> {
    vec![start, vec2(slot.x, (start.y + slot.y) * 0.5), slot]
}

pub fn challenge_path(index: usize) -> Vec<Vec2> {
    let lane = index % 8;
    let wave = index / 8;
    let from_left = (index % 2) == 0;
    let start_x = if from_left {
        -80.0 - (wave as f32) * 20.0
    } else {
        SCREEN_WIDTH + 80.0 + (wave as f32) * 20.0
    };
    let sign = if from_left { 1.0 } else { -1.0 };
    let center_y = 100.0 + lane as f32 * 26.0;
    let end_x = if from_left {
        SCREEN_WIDTH + 100.0
    } else {
        -100.0
    };

    vec![
        vec2(start_x, -40.0 - wave as f32 * 24.0),
        vec2(SCREEN_WIDTH * 0.5 - sign * 100.0, center_y),
        vec2(SCREEN_WIDTH * 0.5 + sign * 100.0, center_y + 90.0),
        vec2(end_x, SCREEN_HEIGHT + 60.0),
    ]
}

pub fn sample_path(points: &[Vec2], t: f32) -> Vec2 {
    if points.is_empty() {
        return Vec2::ZERO;
    }
    if points.len() == 1 {
        return points[0];
    }

    let n = points.len() - 1;
    let clamped = t.clamp(0.0, 1.0);
    let segment_f = clamped * n as f32;
    let mut segment = segment_f.floor() as usize;
    if segment >= n {
        segment = n - 1;
    }
    let local_t = segment_f - segment as f32;

    points[segment].lerp(points[segment + 1], local_t)
}
