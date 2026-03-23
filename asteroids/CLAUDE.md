# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo run            # debug build + run (must run from repo root so `assets/` is found)
cargo run --release  # optimized build
cargo build          # compile only
cargo clippy         # lint
```

No test suite exists. The game is a graphical application — verify changes by running.

## What This Is

An Asteroids clone in Rust using **macroquad 0.4.14** (rendering, audio, input) with Rust edition 2024. No ECS library — entities are plain structs owned directly by `Game`. Window is 800x600.

## Architecture

**Game loop** (`main.rs`): loads assets, starts music, then runs a simple loop — capture input, `game.update(dt)`, `game.draw()`, `next_frame().await`.

**`Game` struct** (`game.rs`): owns all game state. `update()` handles input, entity updates, collision checks, scoring, death/respawn, and particle spawning. `draw()` renders starfield (unshaken), then entities/particles under a camera shake offset, then HUD with the default camera.

**Entity pattern**: each entity type (Player, Enemy, Asteroid, Laser, Pickup) follows the same pattern:
- Owns a `Transform` (position, rotation, scale), a `Sprite` (texture + draw), and a collider (`BoxCollider` or `CircleCollider`)
- Implements the `Collidable` trait which returns a `Collider` enum (either `Obb` or `Circle`)
- Has `alive: bool` for lifetime management — dead entities are `.retain()`-ed out of Vecs or skipped

**Collision** (`collidable.rs`): `overlaps(a, b)` dispatches on `Collider` enum — supports OBB-vs-OBB (SAT), Circle-vs-Circle, and Circle-vs-OBB. `draw_debug` renders wireframe colliders (toggle with F1).

**Collider types**:
- `BoxCollider` + `Obb` (`box_collider.rs`): oriented bounding box. `BoxCollider` stores scale factors and generates an `Obb` from a `Transform` + base texture size.
- `CircleCollider` + `Circle` (`circle_collider.rs`): same pattern — scale factor generates a `Circle` from transform + texture size.

**Effects** (no gameplay logic):
- `ParticleSystem` (`particles.rs`): bursts, flashes, trails, thrust exhaust
- `ScreenShake` (`screen_shake.rs`): trauma-based (squared intensity, linear decay)
- `Starfield` (`starfield.rs`): parallax scrolling background

**Assets** (`assets.rs`): `Assets::load()` loads all textures and sounds from the `assets/` directory at startup. Textures and sounds are cloned into entities that need them (macroquad `Texture2D` and `Sound` are NOT Copy).

**Input** (`input.rs`): `InputState::capture()` snapshots keyboard state each frame. Movement uses WASD/arrows, shoot is Space, quit is Q.

## macroquad 0.4.14 API Notes

- `Texture2D` and `Sound` are NOT Copy — pass by reference (`&Texture2D`) or `.clone()`.
- `draw_texture`, `draw_texture_ex` take `&Texture2D`. `play_sound` takes `&Sound`.
- `Vec2`, `Color`, `Rect` ARE Copy.
- Audio: `use macroquad::audio::{play_sound, PlaySoundParams, Sound, load_sound}`.
- Random: `macroquad::rand::gen_range(min, max)` — not the `rand` crate.
