# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo run            # build and launch the game window
cargo build          # compile without running
cargo clippy         # lint (treat as the primary code-quality check)
```

There are no tests in this project. The game is a graphical application — verify changes by running it.

## What This Is

A simple 2D game template built with **macroquad 0.4.14** (Rust edition 2024). It uses a flat `Vec<Entity>` architecture (not an ECS) with plain structs and free functions for systems. The intent is a minimal starting point for small arcade-style games.

## Architecture

**Game loop** (`main.rs`): `Game::new().await` → fixed loop of `game.update()` / `game.draw()` / `next_frame().await`. Escape quits, P pauses, F1 toggles debug overlay.

**Core struct** (`game.rs`): `Game` owns `Vec<Entity>`, `GameState`, and `AssetManager`. `update()` runs input → physics → collision; `draw()` renders entities then overlays.

**Entities** (`entities.rs`): Each `Entity` has an `id: u32`, `kind: EntityKind`, position/size/velocity, optional texture name, and `active` flag. No inheritance — `EntityKind` enum discriminates behavior.

**Systems** (`systems.rs`): Free functions that take `&mut [Entity]` or `&[Entity]`:
- `apply_player_input` — reads keyboard, sets player velocity
- `check_collisions` — brute-force AABB, returns `Vec<(u32, u32)>` ID pairs
- `clamp_to_screen` — keeps entity within window bounds

**Rendering** (`render.rs`): `draw_entity` draws texture (or magenta fallback) + debug hitbox. `draw_overlay` shows PAUSED/GAME OVER text.

**Assets** (`assets.rs`): `AssetManager` loads from `assets/assets.json` manifest — string-keyed `HashMap<String, Texture2D>` and `HashMap<String, Sound>`. Look up via `assets.texture("id")` / `assets.sound("id")`. All textures get `FilterMode::Nearest`.

**Asset manifest** (`assets/assets.json`): Add new textures/sounds here with `{ "id", "name", "path" }` entries. The `id` field is the lookup key used in code.

## macroquad 0.4.14 API Notes

- `Texture2D` and `Sound` are **not Copy** — pass by reference (`&Texture2D`, `&Sound`).
- `Vec2`, `Color`, `Rect` are Copy.
- Audio imports: `use macroquad::audio::{play_sound, play_sound_once, PlaySoundParams, Sound, load_sound}` (separate from prelude).
- Use `macroquad::rand::gen_range(min, max)` for randomness (avoids `rand` crate conflicts).
