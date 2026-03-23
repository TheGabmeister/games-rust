# macroquad_fancy — 2D Graphics Showcase Spec

A demoscene-style autonomous showcase of macroquad 0.4.14's 2D graphics capabilities. Six procedural scenes cycle with effect-based transitions and a full post-processing stack. No textures, no audio — pure primitives, math, and GLSL shaders.

## Core Behavior

- **Fullscreen borderless** window
- **6 scenes**, ~10 seconds each, fixed order, infinite loop
- **Fully autonomous** — no user input, plays like a demoscene intro
- **Uncapped framerate**, maximize visual density
- **UI overlay**: scene name (fades during transitions) + FPS counter

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

## Architecture

```
src/
├── main.rs           — fullscreen Conf, game loop
├── demo.rs           — DemoRunner state machine (Playing / TransitionOut)
├── scene.rs          — trait Scene { init, update, draw, name }
├── shaders.rs        — all GLSL source strings as const &str
├── palette.rs        — per-scene curated Palette structs
├── postfx.rs         — PostFxPipeline (render targets + material passes)
├── transitions.rs    — TransitionSystem (4 shader materials)
└── scenes/
    ├── mod.rs
    ├── starfield.rs
    ├── fire.rs
    ├── spirograph.rs
    ├── moire.rs
    ├── aurora.rs
    └── voronoi.rs
```

**Scene trait:**
```rust
pub trait Scene {
    fn init(&mut self);
    fn update(&mut self, t: f32, dt: f32);
    fn draw(&self);
    fn name(&self) -> &str;
}
```

**DemoRunner states:**
- `Playing` — current scene updates/draws for ~10s
- `TransitionOut { next_scene }` — both scenes update/draw to separate render targets, transition shader composites, ~1.5s

**Render flow per frame:**
1. Scene(s) draw into render target(s)
2. Transition composite (if transitioning)
3. Post-FX chain (ping-pong)
4. Overlay text to screen

## Dependencies

- `macroquad = "0.4.14"` (existing)
- `noise` — Perlin/simplex noise for organic effects (aurora, fire, dissolve)
