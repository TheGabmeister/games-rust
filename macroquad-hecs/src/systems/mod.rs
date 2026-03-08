use macroquad::audio::{play_sound, PlaySoundParams};
use macroquad::prelude::*;
use hecs::*;
use ::rand::RngExt;

use crate::components::*;
use crate::resources::{Resources, SoundId};

// ── Constants ─────────────────────────────────────────────────────────────────

const PLAYER_SPEED:  f32 = 200.0;
const BULLET_SPEED:  f32 = 500.0;
const BULLET_DAMAGE: i32 = 25;
const BULLET_LIFE:   f32 = 2.0;   // seconds
const HIT_RADIUS:    f32 = 20.0;  // pixels, projectile vs enemy

const PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];

// ── Helpers ───────────────────────────────────────────────────────────────────

fn manhattan_dist(a: Vec2, b: Vec2) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as i32
}

// ── Spawning ──────────────────────────────────────────────────────────────────

pub fn batch_spawn_entities(world: &mut World, n: usize) {
    let mut rng = ::rand::rng();
    let to_spawn = (0..n).map(|_| {
        let pos    = Position(vec2(rng.random_range(0.0..800.0), rng.random_range(0.0..600.0)));
        let vel    = Velocity(Vec2::ZERO);
        let speed  = Speed(rng.random_range(50.0..200.0));
        let hp     = Health(rng.random_range(30..50));
        let dmg    = Damage(rng.random_range(1..10));
        let kc     = KillCount(0);
        let tint   = PALETTE[rng.random_range(0..PALETTE.len())];
        let sprite = Sprite { texture: TextureId::EnemyBlack, tint };
        (pos, vel, speed, hp, dmg, kc, Enemy, sprite, DrawLayer(LAYER_ENEMY))
    });
    world.spawn_batch(to_spawn);
}

/// Player no longer carries Texture2D directly — the Sprite component holds a
/// TextureId and `system_draw` resolves it through Resources at render time.
pub fn spawn_player(world: &mut World) {
    world.spawn((
        Position(vec2(400.0, 300.0)),
        Velocity(Vec2::ZERO),
        Player,
        Sprite { texture: TextureId::PlayerShip, tint: WHITE },
        DrawLayer(LAYER_PLAYER),
    ));
}

// ── Input systems ─────────────────────────────────────────────────────────────

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

// ── Movement systems ──────────────────────────────────────────────────────────

/// Set a random Velocity each frame for every NPC with a Speed component.
/// Player has no Speed, so it is excluded automatically.
pub fn system_wander_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Velocity, &Speed)>,
) {
    let mut rng = ::rand::rng();
    for (vel, speed) in query.query_mut(world) {
        vel.0 = vec2(
            rng.random_range(-speed.0..speed.0),
            rng.random_range(-speed.0..speed.0),
        );
    }
}

/// Integrate Velocity into Position for every entity that has both.
pub fn system_integrate_velocity(
    world: &mut World,
    query: &mut PreparedQuery<(&mut Position, &Velocity)>,
) {
    let dt = get_frame_time();
    for (pos, vel) in query.query_mut(world) {
        pos.0 += vel.0 * dt;
    }
}

// ── Combat systems ────────────────────────────────────────────────────────────

/// Entities with Health+Damage find the nearest living neighbour in range and
/// attack it. O(n²) — acceptable up to a few hundred entities.
pub fn system_fire_at_closest(world: &mut World) {
    for (id0, pos0, dmg0, kc0) in
        &mut world.query::<With<(Entity, &Position, &Damage, &mut KillCount), &Health>>()
    {
        if world.get::<&Health>(id0).map_or(true, |hp| hp.0 <= 0) { continue; }

        const ATTACK_RANGE: i32 = 80;

        let closest = world
            .query::<With<(Entity, &Position), &Health>>()
            .iter()
            .filter(|(id1, _)| *id1 != id0)
            .filter(|(_, pos1)| manhattan_dist(pos0.0, pos1.0) <= ATTACK_RANGE)
            .min_by_key(|(_, pos1)| manhattan_dist(pos0.0, pos1.0))
            .map(|(entity, _)| entity);

        let closest = match closest { Some(e) => e, None => continue };

        let mut hp1 = world.get::<&mut Health>(closest).unwrap();
        if hp1.0 > 0 {
            hp1.0 -= dmg0.0;
            if hp1.0 <= 0 { kc0.0 += 1; }
        }
    }
}

/// Circle-vs-circle collision: projectiles damage the first enemy they touch,
/// then despawn. Two passes — collect then mutate — to satisfy the borrow checker.
pub fn system_projectile_collision(world: &mut World, res: &mut Resources) {
    // Pass 1: read all positions immutably, collect hits.
    // Each QueryBorrow is dropped at the end of its statement.
    let projs: Vec<(Entity, Vec2, i32)> = world
        .query::<With<(Entity, &Position, &Damage), &Projectile>>()
        .iter()
        .map(|(e, pos, dmg)| (e, pos.0, dmg.0))
        .collect();

    let enemies: Vec<(Entity, Vec2)> = world
        .query::<With<(Entity, &Position), &Enemy>>()
        .iter()
        .map(|(e, pos)| (e, pos.0))
        .collect();

    // Collect (projectile, enemy, damage) pairs — one hit per projectile.
    let mut hits: Vec<(Entity, Entity, i32)> = Vec::new();
    for &(proj_e, proj_pos, dmg) in &projs {
        for &(enemy_e, enemy_pos) in &enemies {
            if proj_pos.distance(enemy_pos) < HIT_RADIUS {
                hits.push((proj_e, enemy_e, dmg));
                break; // one hit per projectile
            }
        }
    }

    // Pass 2: apply damage and despawn the projectile.
    for (proj_e, enemy_e, dmg) in hits {
        if let Ok(mut hp) = world.get::<&mut Health>(enemy_e) {
            hp.0 -= dmg;
        }
        let _ = world.despawn(proj_e);
        res.queue_sound(SoundId::Bump);
    }
}

// ── Lifetime / cleanup systems ────────────────────────────────────────────────

/// Tick down Lifetime for every entity that has one (projectiles).
pub fn system_tick_lifetime(world: &mut World, query: &mut PreparedQuery<&mut Lifetime>) {
    let dt = get_frame_time();
    for lt in query.query_mut(world) {
        lt.0 -= dt;
    }
}

/// Despawn entities whose Lifetime has expired.
pub fn system_remove_expired(world: &mut World) {
    let to_remove: Vec<Entity> = world
        .query::<(Entity, &Lifetime)>()
        .iter()
        .filter(|(_, lt)| lt.0 <= 0.0)
        .map(|(e, _)| e)
        .collect();
    for e in to_remove {
        let _ = world.despawn(e);
    }
}

/// Despawn every entity with Health ≤ 0 and credit the score.
pub fn system_remove_dead(world: &mut World, res: &mut Resources) {
    let to_remove: Vec<Entity> = world
        .query::<(Entity, &Health)>()
        .iter()
        .filter(|(_, hp)| hp.0 <= 0)
        .map(|(e, _)| e)
        .collect();
    res.score += to_remove.len() as u32;
    for e in to_remove {
        let _ = world.despawn(e);
    }
}

// ── Audio system ──────────────────────────────────────────────────────────────

/// Drain the audio queue and play each pending sound.
/// Collecting first avoids a simultaneous mutable borrow of audio_queue
/// and immutable borrow of assets in the same expression.
pub fn system_audio(res: &mut Resources) {
    let queue: Vec<SoundId> = res.audio_queue.drain(..).collect();
    for id in queue {
        let sound = match id {
            SoundId::Laser => &res.assets.sfx_laser,
            SoundId::Bump  => &res.assets.sfx_bump,
            SoundId::Lose  => &res.assets.sfx_lose,
        };
        play_sound(sound, PlaySoundParams { looped: false, volume: 0.7 });
    }
}

// ── Render system ─────────────────────────────────────────────────────────────

/// Unified sprite pass: collect all (DrawLayer, Position, Sprite) entities,
/// sort by layer, then draw. This guarantees player always renders on top of
/// enemies, and enemies on top of projectiles, regardless of spawn order.
pub fn system_draw(world: &World, res: &Resources) {
    // Collect owned data so we can sort without holding a QueryBorrow.
    let mut drawables: Vec<(u8, Vec2, TextureId, Color)> = world
        .query::<(&DrawLayer, &Position, &Sprite)>()
        .iter()
        .map(|(layer, pos, sprite)| (layer.0, pos.0, sprite.texture, sprite.tint))
        .collect();

    drawables.sort_unstable_by_key(|&(layer, _, _, _)| layer);

    for (_, pos, tex_id, tint) in drawables {
        let tex = res.texture(tex_id); // &Texture2D
        let w   = tex.width();
        let h   = tex.height();
        draw_texture(tex, pos.x - w / 2.0, pos.y - h / 2.0, tint);
    }

    // HP / kill-count labels — drawn on top of all sprites.
    for (pos, hp, kc) in world.query::<(&Position, &Health, &KillCount)>().iter() {
        draw_text(
            &format!("HP:{} K:{}", hp.0, kc.0),
            pos.0.x - 10.0,
            pos.0.y - tex_label_offset(res),
            14.0,
            WHITE,
        );
    }
}

fn tex_label_offset(res: &Resources) -> f32 {
    res.texture(TextureId::EnemyBlack).height() / 2.0 + 6.0
}
