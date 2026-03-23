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

Space shooter game/template using **macroquad 0.4.14** + **hecs 0.11.0** ECS + **egui-macroquad 0.17.3** + **rand 0.10.0**. Rust edition 2024.

### Core loop

`main.rs` runs a fixed-timestep loop. `Game` owns a `hecs::World` and a `Resources` struct. Three separate phases per frame:
1. `game.capture_input()` — called once before the fixed-step loop
2. `game.update(FIXED_DT)` — called at 60 Hz (may run multiple times per frame)
3. `game.draw()` — called once per frame at variable rate

### System execution order (in `game.rs`)

**`update(dt)`:**
1. Debug toggle (F1)
2. **Playing state only:**
   - `system_tick_powerups`
   - `system_animate` → `system_anim_demo`
   - `system_player_movement` → `system_player_fire`
   - `system_enemy_movement` → `system_enemy_fire`
   - `system_integrate`
   - `system_cull_offscreen` → `system_lifetime` → **`system_apply_despawns`**
   - `system_collision`
   - `system_process_events` → **`system_apply_despawns`** (second pass)
3. `input.clear_transients()`

`system_apply_despawns` runs **twice** per frame — after lifetime/culling and after event processing.

**`draw()`:**
1. `render::draw` (sprites sorted by `DrawLayer`)
2. `render::draw_hud`
3. Debug mode only (`#[cfg(debug_assertions)]`): collider wireframes + egui entity window + `egui_macroquad::draw()`

### Resource layout

`Resources` groups singleton state into domain managers:
- `Assets` — `HashMap<TextureId, Texture2D>` and `HashMap<SfxId/MusicId, Sound>`, loaded from `asset_manifest.rs` paths
- `AnimationDb` — immutable database of `SpriteSheetDef` and `AnimClip` entries, built from `anim_manifest.rs` at startup
- `GameDirector` — score, lives, high score, `GameState` (Playing/Won/Lost), debug_mode; pure game logic, no audio coupling
- `SfxManager` / `MusicManager` — thin wrappers around macroquad audio, only accessed by audio event handlers
- `InputState` — per-frame keyboard snapshot (move_axis, fire_held, etc.)
- `EventQueue` — type-erased deferred event queue (emit during systems, dispatched in `system_process_events`)
- `EventRegistry` — maps event types to handler functions via `TypeId`, registered at startup
- `DespawnQueue` — entities to remove, applied by `system_apply_despawns`

### Observer event system

Uses a **trait-based observer/listener pattern** with type-erased dispatch. Key files: `events.rs` (infrastructure), `handlers.rs` (handler functions), `systems/process_events.rs` (dispatch loop).

**Adding a new event:**
1. Define a struct in `events.rs` + `impl GameEvent` for it
2. Write a handler function in `handlers.rs` taking `(&YourEvent, &mut EventContext)`
3. Register with `event_registry.on::<YourEvent>(handlers::your_handler)` in `Game::new()`

**How it works:**
- Each event is its own struct implementing the `GameEvent` marker trait (not an enum variant)
- `EventQueue` stores type-erased `Box<dyn Any>` events keyed by `TypeId`
- `EventRegistry` maps `TypeId` → `Vec<Box<dyn Fn>>` of handler closures (registered at startup, immutable during dispatch)
- `EventContext` bundles `&mut World`, `&mut GameDirector`, `&mut DespawnQueue`, `&mut SfxManager`, `&mut MusicManager` so all handlers have the same signature
- Handlers can emit follow-up events via `ctx.emit(...)` — these are appended to the dispatch queue after the current event's handlers finish
- Multiple handlers can independently observe the same event type (fan-out)

**Audio is fully decoupled** — gameplay handlers emit `PlaySfx { id }` / `PlayMusic { id }` events instead of calling audio managers directly. Dedicated `on_play_sfx` and `on_play_music` handlers are the only code that touches `SfxManager`/`MusicManager`.

**Borrow-checker design:** `EventQueue` and `EventRegistry` are separate fields on `Resources` to avoid conflicting borrows during dispatch (queue is drained before handlers run; registry is borrowed immutably).

### Sprite sheet animation system

UV-based atlas animation using uniform-grid sprite sheets (e.g. Aseprite exports). Key files: `animation.rs` (data types), `managers/animation_db.rs` (shared database), `anim_manifest.rs` (declarative setup), `systems/animation.rs` (tick systems).

**Architecture:** Definitions (shared, immutable) live in `AnimationDb`; per-entity playback state lives in `Animator` + `SpriteRegion` components. Static sprites (no `SpriteRegion`) render unchanged — the render system uses `Option<&SpriteRegion>` in its query.

**Adding a new sprite sheet animation:**
1. Add a `TextureId` variant for the sheet texture + path in `asset_manifest.rs`
2. Add a `SpriteSheetId` variant in `animation.rs`
3. Add any new `AnimClipName` variants needed in `animation.rs`
4. Register the sheet and clips in `anim_manifest.rs` via `build_animation_db()`
5. Spawn entities with `Sprite::new(TextureId::Sheet)` + `Animator::new(sheet, clip, &anim_db)` + `SpriteRegion::new(sheet, clip, &anim_db)`

**How clips map to the sheet:** Frames are indexed left-to-right, top-to-bottom. `first_frame = row * columns + col`. A clip with `first_frame: 4, frame_count: 4` on a 4-column sheet plays row 1, columns 0–3.

**Changing animations at runtime:** Call `animator.play(AnimClipName::Walk, &anim_db)` — no-op if already playing that clip. `system_animate` updates `SpriteRegion.source` each tick.

### Entity spawning

`prefabs.rs` has factory functions (`spawn_player`, `spawn_enemy`, `spawn_player_bullet`, `spawn_old_hero`, etc.) that bundle components into archetypes. Animated prefabs take `&AnimationDb` to initialize `Animator` and `SpriteRegion`. Restart uses `world.clear()` + re-spawn.

## Key hecs patterns

- **`query_mut` yields components only** — `for (t, v) in world.query_mut::<(&mut Transform, &Velocity)>()` — no entity ID
- **`query().iter()` for entity access** — `for (entity, t) in world.query::<(Entity, &Transform)>().iter()`
- Both yield **flat tuples**, never nested like `(Entity, (&T, &U))`
- **`Option<&T>` in queries** — hecs 0.11 supports optional components: `query::<(&A, Option<&B>)>()` yields `(&A, Option<&B>)` — entities match whether or not they have `B`
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

## Debug tooling

Press **F1** in debug builds to toggle: collider wireframes (green boxes, yellow circles) and an egui entity inspector window listing all live entities by type. Uses `egui-macroquad` for the UI overlay.

## Game tuning

All gameplay constants (speeds, fire rates, scores, screen size) live in `constants.rs`.
