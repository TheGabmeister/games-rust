# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # compile
cargo run            # compile and run the game window
cargo clippy         # lint
```

No test suite exists yet. This is a Rust 2024 edition project.

## Project Overview

A macroquad 0.4.14 game project (currently a minimal scaffold with a single window). The entry point is `src/main.rs` using `#[macroquad::main(window_conf)]` for window configuration (800x600).

## macroquad 0.4.14 API Pitfalls

- `Texture2D` and `Sound` are NOT `Copy` — pass by reference (`&Texture2D`, `&Sound`). `Vec2`, `Color`, `Rect` ARE `Copy`.
- Audio imports are separate: `use macroquad::audio::{play_sound, stop_sound, PlaySoundParams, Sound, load_sound}`.
- Use `FilterMode::Nearest` on textures for crisp pixel art.
- Use `macroquad::rand::gen_range(min, max)` instead of the `rand` crate to avoid dependency conflicts.
