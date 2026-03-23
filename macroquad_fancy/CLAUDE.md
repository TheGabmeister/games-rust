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

A demoscene-style 2D graphics showcase using macroquad 0.4.14 + noise 0.9. Fullscreen, autonomous (no user input), 6 procedural scenes cycle with effect-based transitions and a full post-processing stack. No textures, no audio — pure primitives, math, and GLSL shaders.

## Core Behavior

- **1600x900** window
- **6 scenes**, ~10 seconds each, fixed order, infinite loop
- **Fully autonomous** — no user input, plays like a demoscene intro
- **Uncapped framerate**, maximize visual density
- **UI overlay**: scene name (fades during transitions) + FPS counter

## Architecture

**Render flow each frame:**
1. `DemoRunner` draws current scene → `rt_current` render target
2. During transitions: also draws next scene → `rt_next`, composites via transition shader → `rt_composite`
3. `PostFxPipeline` applies 5 fullscreen shader passes (bloom → chromatic aberration → wave → CRT → vignette) using ping-pong between `rt_a`/`rt_b`, final pass draws to screen
4. Overlay (scene name + FPS) drawn directly to screen

**State machine** (`demo.rs`): `Playing` (scene runs ~10s) → `TransitionOut` (both scenes render for ~1.5s) → `Playing` (next scene). Loops infinitely in fixed order. Transition type index increments mod 4 each cycle.

**Scene trait** (`scene.rs`): `init()` resets state on entry, `update(t, dt)` where `t` is scene-local elapsed time, `draw(&self)` is read-only. During transitions both scenes receive `update()` calls with independent `t` counters.

```rust
pub trait Scene {
    fn init(&mut self);
    fn update(&mut self, t: f32, dt: f32);
    fn draw(&self);
    fn name(&self) -> &str;
}
```

**Key modules:**
- `shaders.rs` — all GLSL ES 100 source as `const &str` (vertex, 8 postfx, 4 transition, 3 scene-specific)
- `palette.rs` — 6 curated `const Palette` structs (background + colors)
- `postfx.rs` — ping-pong render target chain with 8 materials
- `transitions.rs` — 4 transition materials (dissolve, radial wipe, pixelate, slide)
- `scenes/` — one file per scene, each implements `Scene` trait

## Scenes

### 1. Starfield Warp
~800 stars in 3D cylinder space projected to 2D. Speed oscillates slow→fast→slow via cosine curve. Each star drawn as a `draw_line` trail, thickness and brightness scale with depth. Stars respawn at far z when they pass the camera.

**Palette:** Deep space `#000011`, stars white-to-ice-blue (`#AACCFF` → `#FFFFFF`).

### 2. Fire Particles
~2000 particles emitted from bottom center with upward velocity and lateral sine shimmer. Color interpolated via HSL from red (dying) to yellow (fresh). Rendered with **additive blend material** for natural glow overlap. ~200 smaller ember particles with longer lifetime. Scene-local heat shimmer shader distorts UVs.

**Palette:** Black background. Fire: `#FFFF44`, `#FF8800`, `#FF4400`, `#CC1100`, `#440000`.

### 3. Spirograph (Hypotrochoid)
3-4 overlapping hypotrochoid curves using parametric equations `x = (R-r)cos(t) + d·cos(t(R-r)/r)`. Parameter `max_t` grows over time creating a drawing-in effect. Older segments fade via alpha gradient. Drawn as polyline segments with `draw_line`.

**Palette:** Dark navy `#0A0A2E`. Curves: neon cyan `#00FFCC`, magenta `#FF00FF`, gold `#FFD700`, electric blue `#00AAFF`.

### 4. Moiré Patterns
Two sets of ~60 concentric circles (`draw_circle_lines`) with centers orbiting slowly on independent Lissajous paths. Each set drawn at 40-50% alpha. Interference patterns emerge naturally from overlap.

**Palette:** Black `#000000` background. Circles: white `#FFFFFF` at alpha 0.4.

### 5. Aurora Borealis
4-5 curtains of light, each defined by a sum-of-sines wavy top edge. Per x-column, vertical gradient strips fade from full color to transparent. Rendered with **additive blend** so overlapping curtains brighten. Background: 100 dim star dots.

**Palette:** Dark sky `#050520`. Greens `#00FF88` / `#00CC66`, blues `#4488FF` / `#2244CC`, purples `#8844FF` / `#6622CC`.

### 6. Voronoi Shatter (GPU)
20-30 seed points moving with bounce-off-edges physics on CPU. A **GPU fragment shader** computes per-pixel nearest-seed lookup for real-time Voronoi cells. Edge detection via `second_dist - min_dist` with smoothstep. Seed positions passed as uniform array.

**Palette:** Charcoal `#111111` edges. Cells: jewel tones `#FF3366`, `#33CCFF`, `#FFCC00`, `#66FF66`, `#CC66FF`, `#FF6633`.

## Transitions

4 effect types, rotating in order. Each is a fullscreen shader compositing two scene render targets with a `progress` uniform (0.0 → 1.0 over ~1.5 seconds):

1. **Dissolve** — hash-based noise threshold; pixels flip from scene A to B as noise < progress
2. **Radial Wipe** — angle from screen center compared to `progress × 2π`
3. **Pixelate** — block size increases then decreases; scene swaps at midpoint
4. **Slide** — horizontal push, next scene slides in from right

## Post-Processing Stack

Multi-pass ping-pong between two fullscreen render targets. All shaders GLSL ES 100. Applied every frame after scene/transition rendering:

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
