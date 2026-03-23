# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # compile
cargo run            # compile and run (fullscreen)
cargo clippy         # lint
```

No test suite. Rust 2024 edition. Verify changes visually by running.

## Project Overview

A demoscene-style 2D graphics showcase using macroquad 0.4.14 + noise 0.9. Fullscreen, autonomous (no user input), 6 procedural scenes cycle with effect-based transitions and a full post-processing stack. No textures, no audio — pure primitives, math, and GLSL shaders. See SPEC.md for the full design spec.

## Architecture

**Render flow each frame:**
1. `DemoRunner` draws current scene → `rt_current` render target
2. During transitions: also draws next scene → `rt_next`, composites via transition shader → `rt_composite`
3. `PostFxPipeline` applies 5 fullscreen shader passes (bloom → chromatic aberration → wave → CRT → vignette) using ping-pong between `rt_a`/`rt_b`, final pass draws to screen
4. Overlay (scene name + FPS) drawn directly to screen

**State machine** (`demo.rs`): `Playing` (scene runs ~10s) → `TransitionOut` (both scenes render for ~1.5s) → `Playing` (next scene). Loops infinitely in fixed order. Transition type index increments mod 4 each cycle.

**Scene trait** (`scene.rs`): `init()` resets state on entry, `update(t, dt)` where `t` is scene-local elapsed time, `draw(&self)` is read-only. During transitions both scenes receive `update()` calls with independent `t` counters.

**Key modules:**
- `shaders.rs` — all GLSL ES 100 source as `const &str` (vertex, 8 postfx, 4 transition, 3 scene-specific)
- `palette.rs` — 6 curated `const Palette` structs (background + colors)
- `postfx.rs` — ping-pong render target chain with 8 materials
- `transitions.rs` — 4 transition materials (dissolve, radial wipe, pixelate, slide)
- `scenes/` — one file per scene, each implements `Scene` trait

## macroquad 0.4.14 API Pitfalls

- `Texture2D` and `Sound` are NOT `Copy` — clone or pass by reference. `Vec2`, `Color`, `Rect` ARE `Copy`.
- `gen_range` requires explicit import: `use macroquad::rand::gen_range;` (not in prelude).
- `hsl_to_rgb` requires: `use macroquad::color::hsl_to_rgb;` (not in prelude).
- `BlendState`, `BlendFactor`, `BlendValue`, `Equation` require: `use macroquad::window::miniquad::*;` (not in prelude).
- Render target camera setup: use `Camera2D::from_display_rect(Rect::new(0., 0., w, h))` with `.render_target = Some(rt.clone())`.
- Fullscreen pass pattern: `set_camera → gl_use_material → draw_texture_ex(source, 0, 0, WHITE, {dest_size}) → gl_use_default_material`.
- Uniform arrays: declare with `UniformDesc::array(UniformDesc::new("name", type), count)`, set with `mat.set_uniform_array("name", &array)`. Types must match exactly.
- Second texture binding for shaders: declare in `MaterialParams { textures: vec!["Texture2".into()] }`, set with `mat.set_texture("Texture2", tex.clone())`.
- Additive blending: `BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)` in `PipelineParams`.
