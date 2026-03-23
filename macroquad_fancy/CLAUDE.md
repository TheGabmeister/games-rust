# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # compile
cargo run            # compile and run
cargo clippy         # lint
```

No test suite. Rust 2024 edition. Verify changes visually by running.

## Project Overview

A demoscene-style 2D graphics showcase using macroquad 0.4.14 + noise 0.9. Interactive — user navigates between 6 procedural scenes via buttons and toggles post-processing effects on/off. No textures, no audio — pure primitives, math, and GLSL shaders.

## Core Behavior

- **1600x900** window
- **6 scenes**, manually navigated via Prev/Next buttons (no auto-cycling)
- **Interactive UI**: scene navigation buttons (bottom-center), post-FX toggle buttons (top-right), FPS counter (top-left)
- **Uncapped framerate**, maximize visual density

## Architecture

**Render flow each frame:**
1. `DemoRunner` draws current scene → `rt_current` render target
2. `PostFxPipeline` applies up to 5 toggleable fullscreen shader passes (bloom → chromatic aberration → wave → CRT → vignette) using ping-pong between `rt_a`/`rt_b`, final pass draws to screen
3. UI overlay (buttons + scene name + FPS) drawn directly to screen

**DemoRunner** (`demo.rs`): Owns the scene list, handles mouse click input for button interactions, delegates to `PostFxPipeline`. Scene switching calls `init()` on new scene and resets `scene_time`. UI buttons are hand-drawn with `draw_rectangle` + `draw_text` and hit-tested against mouse position.

**Scene trait** (`scene.rs`): `init()` resets state on entry, `update(t, dt)` where `t` is scene-local elapsed time, `draw(&self)` is read-only.

```rust
pub trait Scene {
    fn init(&mut self);
    fn update(&mut self, t: f32, dt: f32);
    fn draw(&self);
    fn name(&self) -> &str;
}
```

**PostFxPipeline** (`postfx.rs`): Ping-pong render target chain with 8 materials. Each of the 5 effects has a `pub bool` toggle (`bloom_enabled`, `chromatic_enabled`, `wave_enabled`, `crt_enabled`, `vignette_enabled`). The `apply()` method tracks an `in_a` flag to know which render target holds the current result, skipping disabled passes without breaking the chain.

**Key modules:**
- `shaders.rs` — all GLSL ES 100 source as `const &str` (vertex, 8 postfx, 4 transition, 3 scene-specific)
- `palette.rs` — 6 curated `const Palette` structs (background + colors)
- `postfx.rs` — ping-pong render target chain with toggleable effects
- `transitions.rs` — 4 transition materials (currently unused, retained for potential reuse)
- `scenes/` — one file per scene, each implements `Scene` trait

## Scenes

1. **Starfield Warp** — ~800 stars in 3D cylinder space, speed oscillates via cosine, `draw_line` trails
2. **Fire Particles** — ~2000 particles with additive blend material, HSL color interpolation, heat shimmer shader
3. **Spirograph** — 3-4 hypotrochoid curves with parametric drawing-in effect, alpha gradient fade
4. **Moiré Patterns** — two sets of ~60 concentric circles on Lissajous paths, interference from overlap
5. **Aurora Borealis** — 4-5 sum-of-sines light curtains with additive blend, background star dots
6. **Voronoi Shatter** — GPU fragment shader for per-pixel Voronoi cells, CPU-side seed physics

## Post-Processing Stack

Multi-pass ping-pong between two fullscreen render targets. All shaders GLSL ES 100. Each effect individually toggleable:

1. **Bloom** — downsample bright pixels to half-res, 9-tap Gaussian blur (H + V), additive combine back
2. **Chromatic Aberration** — offset R and B channels by ±0.003 in UV space
3. **Wave Distortion** — sinusoidal UV offset `uv.y += sin(uv.y * 30 + time * 2) * 0.002`
4. **CRT** — scanlines + barrel distortion combined in one pass
5. **Vignette + Color Grading** — corner darkening + lift/gamma/gain (final pass to screen)

## Dependencies

- `macroquad = "0.4.14"`
- `noise` — Perlin/simplex noise for organic effects (aurora, fire, dissolve)

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
