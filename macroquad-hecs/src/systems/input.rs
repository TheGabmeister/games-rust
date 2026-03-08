use hecs::*;
use macroquad::prelude::*;

use crate::components::*;
use crate::resources::{InputState, Resources, SoundId};

const PLAYER_SPEED: f32 = 200.0;
const BULLET_SPEED: f32 = 500.0;
const BULLET_LIFE: f32 = 2.0; // seconds

/// Capture key/mouse input once per frame into a singleton resource.
pub fn system_capture_input(input: &mut InputState) {
    let mut move_axis = Vec2::ZERO;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        move_axis.x += 1.0;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        move_axis.x -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        move_axis.y += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        move_axis.y -= 1.0;
    }

    let (mx, my) = mouse_position();
    input.move_axis = move_axis;
    input.aim_screen = vec2(mx, my);
    input.shoot_pressed = is_mouse_button_pressed(MouseButton::Left);
    input.confirm_pressed = is_key_pressed(KeyCode::Enter);
    input.cancel_pressed = is_key_pressed(KeyCode::Escape);
    input.resume_pressed = is_key_pressed(KeyCode::Space);
}

/// Write player velocity from captured input.
/// The integrator applies it to Position in the same frame.
pub fn system_player_move(world: &mut World, input: InputState) {
    for (vel, _) in &mut world.query::<(&mut Velocity, &Player)>() {
        vel.0 = input.move_axis * PLAYER_SPEED;
    }
}

/// Spawn a projectile aimed at the captured mouse cursor on click.
pub fn system_player_shoot(world: &mut World, input: InputState, res: &mut Resources) {
    if !input.shoot_pressed {
        return;
    }

    // Collect player entity + position in one query to avoid a double-borrow.
    let player_info: Option<(Entity, Vec2)> = world
        .query::<With<(Entity, &Position), &Player>>()
        .iter()
        .next()
        .map(|(e, pos)| (e, pos.0));

    let Some((owner, origin)) = player_info else {
        return;
    };

    let dir = (input.aim_screen - origin).normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }

    world.spawn((
        Position(origin),
        Velocity(dir * BULLET_SPEED),
        Lifetime(BULLET_LIFE),
        Projectile { owner },
        Collider::Circle { radius: 4.0 },
        Sprite {
            texture: TextureId::PlayerLaser,
            tint: WHITE,
        },
        DrawLayer(LAYER_PROJECTILE),
    ));

    res.queue_sound(SoundId::Laser);
}
