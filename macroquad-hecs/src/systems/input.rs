use macroquad::prelude::*;
use hecs::*;

use crate::components::*;
use crate::resources::{Resources, SoundId};

const PLAYER_SPEED:  f32 = 200.0;
const BULLET_SPEED:  f32 = 500.0;
const BULLET_DAMAGE: i32 = 25;
const BULLET_LIFE:   f32 = 2.0; // seconds

/// Write player velocity from keyboard input.
/// The integrator applies it to Position the same frame.
pub fn system_player_input(world: &mut World) {
    for (vel, _) in &mut world.query::<(&mut Velocity, &Player)>() {
        vel.0 = Vec2::ZERO;
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { vel.0.x += PLAYER_SPEED; }
        if is_key_down(KeyCode::Left)  || is_key_down(KeyCode::A) { vel.0.x -= PLAYER_SPEED; }
        if is_key_down(KeyCode::Down)  || is_key_down(KeyCode::S) { vel.0.y += PLAYER_SPEED; }
        if is_key_down(KeyCode::Up)    || is_key_down(KeyCode::W) { vel.0.y -= PLAYER_SPEED; }
    }
}

/// Spawn a projectile aimed at the mouse cursor on left-click.
pub fn system_player_shoot(world: &mut World, res: &mut Resources) {
    if !is_mouse_button_pressed(MouseButton::Left) { return; }

    // Collect player entity + position in one query to avoid a double-borrow.
    let player_info: Option<(Entity, Vec2)> = world
        .query::<With<(Entity, &Position), &Player>>()
        .iter()
        .next()
        .map(|(e, pos)| (e, pos.0));

    let Some((owner, origin)) = player_info else { return };

    let (mx, my) = mouse_position();
    let dir = (vec2(mx, my) - origin).normalize_or_zero();
    if dir == Vec2::ZERO { return; }

    world.spawn((
        Position(origin),
        Velocity(dir * BULLET_SPEED),
        Damage(BULLET_DAMAGE),
        Lifetime(BULLET_LIFE),
        Projectile { owner },
        Sprite { texture: TextureId::PlayerLaser, tint: WHITE },
        DrawLayer(LAYER_PROJECTILE),
    ));

    res.queue_sound(SoundId::Laser);
}
