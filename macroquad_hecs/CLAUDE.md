# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # debug build
cargo build --release
cargo run            # launch game (600x800 window)
```

No tests or CI. No linter configured — use `cargo clippy` if needed.

## Architecture

Space shooter game/template using **macroquad 0.4.14** + **hecs 0.11.0** ECS + **rand 0.10.0**. Rust edition 2024.

### Core loop

`main.rs` runs a fixed-timestep loop: `Game::update(dt)` at 60 Hz (`FIXED_DT`), `Game::draw()` every frame. `Game` owns a `hecs::World` and a `Resources` struct.

### System execution order (in `game.rs::update`)

1. `system_capture_input` — always runs
2. **Playing state only:**
   - `system_player_movement` → `system_player_fire`
   - `system_enemy_movement` → `system_enemy_fire`
   - `system_integrate`
   - `system_cull_offscreen` → `system_lifetime` → **`system_apply_despawns`**
   - `system_collision`
   - `system_process_events` → **`system_apply_despawns`** (second pass)
3. **Draw** (variable rate): `render::draw` (sorted by `DrawLayer`) → debug colliders (F1, debug builds only) → `render::draw_hud`

`system_apply_despawns` runs **twice** per frame — after lifetime/culling and after event processing.

### Resource layout

`Resources` groups singleton state into domain managers:
- `Assets` — `HashMap<TextureId, Texture2D>` and `HashMap<SfxId/MusicId, Sound>`, loaded from `asset_manifest.rs` paths
- `GameDirector` — score, lives, high score, `GameState` (Playing/Won/Lost), debug_mode
- `SfxManager` / `MusicManager` — thin wrappers around macroquad audio
- `InputState` — per-frame keyboard snapshot (move_axis, fire_held, etc.)
- `EventBus` — deferred `GameEvent` queue (emit during systems, drain in `system_process_events`)
- `DespawnQueue` — entities to remove, applied by `system_apply_despawns`

### Deferred event pattern

Systems never mutate world state directly for cross-cutting concerns. Instead:
- Collision/gameplay systems **emit** `GameEvent` variants to `EventBus`
- `system_process_events` **drains** the bus and handles scoring, lives, despawns, state transitions
- This avoids borrow-checker conflicts between querying and mutating the world

### Entity spawning

`prefabs.rs` has factory functions (`spawn_player`, `spawn_enemy`, `spawn_player_bullet`, etc.) that bundle components into archetypes. Restart uses `world.clear()` + re-spawn.

## Key hecs patterns

- **`query_mut` yields components only** — `for (t, v) in world.query_mut::<(&mut Transform, &Velocity)>()` — no entity ID
- **`query().iter()` for entity access** — `for (entity, t) in world.query::<(Entity, &Transform)>().iter()`
- Both yield **flat tuples**, never nested like `(Entity, (&T, &U))`
- **Two-pass spawn/despawn** — collect data in a query loop, drop the borrow, then spawn/despawn freely
- **Drop `RefMut` before world mutation** — `world.get::<&mut T>()` returns a guard that blocks `&mut World` calls

## macroquad 0.4.14 API notes

- `Texture2D` and `Sound` are **not Copy** — pass by reference (`&Texture2D`, `&Sound`)
- `Vec2`, `Color`, `Rect` **are Copy**
- Audio requires `features = ["audio"]` and separate import: `use macroquad::audio::*`
- Use `FilterMode::Nearest` on textures for pixel-art rendering
- Use `macroquad::rand::gen_range(min, max)` to avoid conflicts with the `rand` crate

## Collision system

- `BoxCollider { half: Vec2 }` stores **half-extents**, not full size
- `CollisionLayer { member: u32, mask: u32 }` — bitmask filtering; overlap requires **both** `(a.mask & b.member) != 0` and `(b.mask & a.member) != 0`
- Supports AABB/AABB, Circle/Circle, and mixed AABB/Circle overlaps
- Layer constants in `constants.rs`: `LAYER_PLAYER`, `LAYER_PLAYER_BULLET`, `LAYER_ENEMY`, `LAYER_ENEMY_BULLET`, `LAYER_PICKUP`

## Game tuning

All gameplay constants (speeds, fire rates, scores, screen size) live in `constants.rs`.
