use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    DualFighterWingman, Enemy, Player, PrimitiveShape, RenderablePrimitive, TractorBeamPhase,
    TractorBeamState, Transform,
};
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::resources::{GameMode, Resources, StageType};

pub fn draw(world: &World, resources: &Resources) {
    clear_background(Color::from_rgba(7, 10, 22, 255));

    draw_background();
    draw_entities(world);
    draw_dual_wingman(world);
    draw_tractor_beams(world);
    draw_ui(resources);
}

fn draw_background() {
    for i in 0..90 {
        let x = ((i * 73) % SCREEN_WIDTH as i32) as f32;
        let y = ((i * 137) % SCREEN_HEIGHT as i32) as f32;
        let a = 0.12 + (i as f32 % 7.0) * 0.08;
        draw_circle(x, y, 1.2, Color::new(0.9, 0.95, 1.0, a.min(0.8)));
    }
}

fn draw_entities(world: &World) {
    let mut draw_list: Vec<(i32, Entity, Vec2, RenderablePrimitive)> = Vec::new();
    for (entity, transform, primitive) in world
        .query::<(Entity, &Transform, &RenderablePrimitive)>()
        .iter()
    {
        draw_list.push((primitive.layer, entity, transform.pos, *primitive));
    }

    draw_list.sort_by_key(|item| item.0);

    for (_, _entity, pos, primitive) in draw_list {
        draw_primitive(pos, primitive);
    }
}

fn draw_dual_wingman(world: &World) {
    for (transform, primitive, _player, wingman) in world
        .query::<(
            &Transform,
            &RenderablePrimitive,
            &Player,
            &DualFighterWingman,
        )>()
        .iter()
    {
        let wingman_pos = vec2(transform.pos.x + wingman.offset, transform.pos.y);
        draw_primitive(wingman_pos, *primitive);
    }
}

fn draw_tractor_beams(world: &World) {
    for (_enemy, transform, beam) in world
        .query::<(&Enemy, &Transform, &TractorBeamState)>()
        .iter()
    {
        if beam.phase == TractorBeamPhase::Idle {
            continue;
        }

        let alpha = if beam.phase == TractorBeamPhase::Telegraph {
            0.18
        } else {
            0.35
        };

        let top = transform.pos + vec2(0.0, 12.0);
        let left = vec2(top.x - 24.0, SCREEN_HEIGHT - 60.0);
        let right = vec2(top.x + 24.0, SCREEN_HEIGHT - 60.0);
        draw_triangle(top, left, right, Color::new(0.3, 0.8, 1.0, alpha));
        draw_rectangle(
            top.x - 10.0,
            top.y,
            20.0,
            SCREEN_HEIGHT - top.y - 60.0,
            Color::new(0.6, 0.9, 1.0, alpha * 0.7),
        );
    }
}

fn draw_primitive(pos: Vec2, primitive: RenderablePrimitive) {
    match primitive.shape {
        PrimitiveShape::Rect => {
            draw_rectangle(
                pos.x - primitive.size.x * 0.5,
                pos.y - primitive.size.y * 0.5,
                primitive.size.x,
                primitive.size.y,
                primitive.color,
            );
        }
        PrimitiveShape::Circle => {
            draw_circle(pos.x, pos.y, primitive.size.x * 0.5, primitive.color);
        }
        PrimitiveShape::Triangle => {
            let half_w = primitive.size.x * 0.5;
            let half_h = primitive.size.y * 0.5;
            draw_triangle(
                vec2(pos.x, pos.y - half_h),
                vec2(pos.x - half_w, pos.y + half_h),
                vec2(pos.x + half_w, pos.y + half_h),
                primitive.color,
            );
        }
    }
}

fn draw_ui(resources: &Resources) {
    draw_text(
        &format!("SCORE {:07}", resources.score.score),
        14.0,
        28.0,
        30.0,
        WHITE,
    );
    draw_text(
        &format!("HI {:07}", resources.hi_score.value),
        240.0,
        28.0,
        30.0,
        YELLOW,
    );
    draw_text(
        &format!("STAGE {}", resources.stage.number),
        470.0,
        28.0,
        26.0,
        LIGHTGRAY,
    );

    let lives_to_draw = resources.lives.lives.max(0) as usize;
    for i in 0..lives_to_draw {
        let x = 24.0 + i as f32 * 20.0;
        draw_triangle(
            vec2(x, SCREEN_HEIGHT - 24.0),
            vec2(x - 8.0, SCREEN_HEIGHT - 10.0),
            vec2(x + 8.0, SCREEN_HEIGHT - 10.0),
            SKYBLUE,
        );
    }

    let emblem_count = (resources.stage.number as usize).min(16);
    for i in 0..emblem_count {
        let x = SCREEN_WIDTH - 20.0 - i as f32 * 14.0;
        let color = if (i + 1) % 4 == 3 { GOLD } else { DARKGREEN };
        draw_circle(x, SCREEN_HEIGHT - 18.0, 5.0, color);
    }

    match resources.flow.mode {
        GameMode::Attract => {
            draw_center_text("GALAGA", 310.0, 72.0, Color::new(0.95, 0.98, 1.0, 1.0));
            draw_center_text("PRESS ENTER", 380.0, 38.0, SKYBLUE);
        }
        GameMode::Ready => {
            if resources.stage.stage_type == StageType::Challenge {
                draw_center_text("CHALLENGE STAGE", 380.0, 44.0, GOLD);
            } else {
                draw_center_text("READY", 380.0, 44.0, SKYBLUE);
            }
        }
        GameMode::StageClear => {
            draw_center_text("STAGE CLEAR", 380.0, 44.0, GREEN);
            if resources.stage.stage_type == StageType::Challenge {
                draw_center_text(
                    &format!(
                        "HITS: {}/{}",
                        resources.stage.challenge_hits, resources.stage.challenge_total
                    ),
                    430.0,
                    30.0,
                    YELLOW,
                );
            }
        }
        GameMode::PlayerDeath => {
            draw_center_text("FIGHTER LOST", 380.0, 42.0, RED);
        }
        GameMode::Pause => {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                Color::new(0.0, 0.0, 0.0, 0.5),
            );
            draw_center_text("PAUSED", 390.0, 54.0, WHITE);
        }
        GameMode::GameOver => {
            draw_center_text("GAME OVER", 380.0, 54.0, RED);
            draw_center_text("PRESS ENTER TO RESTART", 430.0, 30.0, LIGHTGRAY);
        }
        GameMode::Playing => {}
    }
}

fn draw_center_text(text: &str, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (SCREEN_WIDTH - dims.width) * 0.5, y, size, color);
}
