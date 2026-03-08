use macroquad::prelude::*;
use hecs::*;
use ::rand::RngExt;

mod assets;
use assets::Assets;

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Health(i32);

#[derive(Debug)]
struct Speed(f32);

#[derive(Debug)]
struct Damage(i32);

#[derive(Debug)]
struct KillCount(i32);

struct Tint(Color);

struct Player;

const PLAYER_SPEED: f32 = 200.0;

fn manhattan_dist(x0: f32, x1: f32, y0: f32, y1: f32) -> i32 {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();
    (dx + dy) as i32
}

const PALETTE: [Color; 6] = [RED, GREEN, BLUE, YELLOW, ORANGE, MAGENTA];

fn batch_spawn_entities(world: &mut World, n: usize) {
    let mut rng = ::rand::rng();

    let to_spawn = (0..n).map(|_| {
        let pos = Position {
            x: rng.random_range(0.0..800.0),
            y: rng.random_range(0.0..600.0),
        };
        let s = Speed(rng.random_range(50.0..200.0));
        let hp = Health(rng.random_range(30..50));
        let dmg = Damage(rng.random_range(1..10));
        let kc = KillCount(0);
        let tint = Tint(PALETTE[rng.random_range(0..PALETTE.len())]);

        (pos, s, hp, dmg, kc, tint)
    });

    world.spawn_batch(to_spawn);
    // We could instead call `world.spawn((pos, s, hp, dmg, kc))` for each entity, but `spawn_batch`
    // is faster.
}

fn spawn_player(world: &mut World, texture: Texture2D) {
    world.spawn((
        Position { x: 400.0, y: 300.0 },
        Tint(WHITE),
        Player,
        texture,
    ));
}

fn system_player_input(world: &mut World) {
    let dt = get_frame_time();
    for (pos, _) in &mut world.query::<(&mut Position, &Player)>() {
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { pos.x += PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Left)  || is_key_down(KeyCode::A) { pos.x -= PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Down)  || is_key_down(KeyCode::S) { pos.y += PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Up)    || is_key_down(KeyCode::W) { pos.y -= PLAYER_SPEED * dt; }
    }
}

fn system_integrate_motion(
    world: &mut World,
    query: &mut PreparedQuery<(Entity, &mut Position, &Speed)>,
) {
    let mut rng = ::rand::rng();

    let dt = get_frame_time();
    for (_id, pos, s) in query.query_mut(world) {
        let change = (rng.random_range(-s.0..s.0), rng.random_range(-s.0..s.0));
        pos.x += change.0 * dt;
        pos.y += change.1 * dt;
    }
}

// In this system entities find the closest entity and fire at them
fn system_fire_at_closest(world: &mut World) {
    for (id0, pos0, dmg0, kc0) in
        &mut world.query::<With<(Entity, &Position, &Damage, &mut KillCount), &Health>>()
    {
        // Skip if killed earlier this tick. Borrow is dropped at end of expression.
        if world.get::<&Health>(id0).map_or(true, |hp| hp.0 <= 0) { continue; }

        // Find closest:
        // Nested queries are O(n^2) and you usually want to avoid that by using some sort of
        // spatial index like a quadtree or more general BVH, which we don't bother with here since
        // it's out of scope for the example.
        const ATTACK_RANGE: i32 = 80;

        let closest = world
            .query::<With<(Entity, &Position), &Health>>()
            .iter()
            .filter(|(id1, _)| *id1 != id0)
            .filter(|(_, pos1)| manhattan_dist(pos0.x, pos1.x, pos0.y, pos1.y) <= ATTACK_RANGE)
            .min_by_key(|(_, pos1)| manhattan_dist(pos0.x, pos1.x, pos0.y, pos1.y))
            .map(|(entity, _)| entity);

        let closest = match closest {
            Some(entity) => entity,
            None => continue,
        };

        // Deal damage:
        /*
                // Get target unit hp like this:
                let mut hp1 = world.query_one::<&mut Health>(closest_id.unwrap()).unwrap();
                let hp1 = hp1.get().unwrap();
        */

        // Or like this:
        let mut hp1 = world.get::<&mut Health>(closest).unwrap();

        // Is target unit still alive?
        if hp1.0 > 0 {
            // apply damage
            hp1.0 -= dmg0.0;
            if hp1.0 <= 0 {
                kc0.0 += 1;
            }
        }
    }
}

fn system_remove_dead(world: &mut World) {
    // Here we query entities with 0 or less hp and despawn them
    let mut to_remove: Vec<Entity> = Vec::new();
    for (id, hp) in &mut world.query::<(Entity, &Health)>() {
        if hp.0 <= 0 {
            to_remove.push(id);
        }
    }

    for entity in to_remove {
        world.despawn(entity).unwrap();
    }
}

fn system_draw(world: &World) {
    for (pos, hp, kc, tint) in world.query::<(&Position, &Health, &KillCount, &Tint)>().iter() {
        draw_circle(pos.x, pos.y, 10.0, tint.0);
        draw_text(
            &format!("HP:{} K:{}", hp.0, kc.0),
            pos.x - 10.0,
            pos.y - 15.0,
            16.0,
            WHITE,
        );
    }
    for (pos, tex, _) in world.query::<(&Position, &Texture2D, &Player)>().iter() {
        let w = tex.width();
        let h = tex.height();
        draw_texture(tex, pos.x - w / 2.0, pos.y - h / 2.0, WHITE);
    }
}

#[macroquad::main("Game")]
async fn main() {
    let assets = Assets::load().await;
    let mut world = World::new();

    batch_spawn_entities(&mut world, 50);
    spawn_player(&mut world, assets.player_ship);

    let mut motion_query = PreparedQuery::<(Entity, &mut Position, &Speed)>::default();
    let mut paused = false;

    loop {
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Update
        if !paused {
            system_integrate_motion(&mut world, &mut motion_query);
            system_fire_at_closest(&mut world);
            system_remove_dead(&mut world);
            system_player_input(&mut world);
        }

        // Draw
        clear_background(BLACK);
        system_draw(&world);
        if paused {
            draw_text("PAUSED  [Space] resume  [Esc] quit", 10.0, 20.0, 20.0, YELLOW);
        } else {
            draw_text("[Space] pause  [Esc] quit", 10.0, 20.0, 20.0, GRAY);
        }

        next_frame().await;
    }
}
