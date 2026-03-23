# Scene System Spec

## Context

The game currently has a single hardcoded scene (`spawn_entities()` in `game.rs`). This spec adds a scene system that supports a main menu and multiple gameplay levels loaded from external RON data files, with a campaign file defining level progression.

## Goals

- Main menu scene (hardcoded in Rust) with its own update/draw logic
- Gameplay scenes loaded from `assets/scenes/*.ron` files
- Campaign progression defined in `assets/scenes/campaign.ron`
- Clean scene transitions: entering a new scene despawns all entities from the previous one
- Score and lives persist across scenes within a run; everything else resets per scene
- Player is always spawned fresh in each scene (no entity carryover)
- Game over (lost all lives) returns to main menu
- Clearing the final campaign scene shows the existing `GameState::Won` state
- Stage-clear condition: all enemies destroyed (same as today)

## Non-goals (explicitly out of scope)

- Visual scene transitions (fade/wipe) — code should be structured so these can be added later, but no transition effects are implemented now
- Wave/phase system within scenes
- Save/continue across app restarts
- Per-scene asset loading (all assets still load once at startup)
- Custom win conditions per scene

---

## New dependencies

Add to `Cargo.toml`:
```toml
serde = { version = "1", features = ["derive"] }
ron = "0.9"
```

---

## Data format

### Scene file (`assets/scenes/level_01.ron`)

```ron
(
    name: "Asteroid Belt",
    music: Some(GameMusic),
    background_color: (0.05, 0.05, 0.12, 1.0),
    entities: [
        Enemy(kind: Black, pos: (150.0, 100.0)),
        Enemy(kind: Blue,  pos: (300.0, 150.0)),
        Enemy(kind: Green, pos: (450.0, 100.0)),
        Pickup(kind: Life, pos: (180.0, 220.0)),
        Pickup(kind: Star, pos: (420.0, 220.0)),
        Powerup(effect: Bolt, pos: (260.0, 280.0)),
        Powerup(effect: Shield, pos: (340.0, 280.0)),
    ],
)
```

### Campaign file (`assets/scenes/campaign.ron`)

```ron
(
    scenes: [
        "level_01.ron",
        "level_02.ron",
        "level_03.ron",
    ],
)
```

Scene filenames are relative to `assets/scenes/`.

---

## Rust types

### Scene data (new file: `src/scene.rs`)

```rust
use serde::Deserialize;

/// Raw scene definition deserialized from a .ron file.
#[derive(Deserialize)]
pub struct SceneDef {
    pub name: String,
    pub music: Option<MusicId>,          // reuse existing MusicId enum
    pub background_color: Option<(f32, f32, f32, f32)>,
    pub entities: Vec<EntityDef>,
}

/// One entity to spawn in a scene.
#[derive(Deserialize)]
pub enum EntityDef {
    Enemy  { kind: EnemyKind, pos: (f32, f32) },
    Pickup { kind: PickupKind, pos: (f32, f32) },
    Powerup { effect: PowerupEffect, pos: (f32, f32) },
}
```

Position values are raw pixel coordinates (same coordinate system as the game, origin top-left).

`EnemyKind`, `PickupKind`, `PowerupEffect`, and `MusicId` need `#[derive(Deserialize)]` added to their existing definitions in `components.rs` / `events.rs`. RON natively maps Rust enum variant names, so no custom mapping code is needed.

### Campaign data (new file: `src/campaign.rs`)

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CampaignDef {
    pub scenes: Vec<String>,  // filenames relative to assets/scenes/
}
```

Runtime campaign state (not serialized):

```rust
pub struct Campaign {
    pub def: CampaignDef,
    pub scenes: Vec<SceneDef>,       // all scenes loaded at startup
    pub current_index: usize,
}
```

All scene RON files are loaded and parsed once at startup (during `Game::new()`), stored in `Campaign.scenes`. This avoids async file I/O during scene transitions.

### Active scene enum

```rust
pub enum ActiveScene {
    Menu,
    Gameplay,   // campaign.current_index determines which scene
}
```

This lives on the `Game` struct (or in `Resources`). The menu is hardcoded in Rust; gameplay scenes are data-driven from `Campaign`.

---

## Scene lifecycle

### Loading (startup)

1. `Assets::load()` — unchanged, loads all textures and sounds
2. Load `campaign.ron`, parse it, then load and parse each referenced `.ron` scene file
3. Store `Campaign` in `Resources` (new field)
4. Set `ActiveScene::Menu`

### Main menu

- **Update:** Check for input (e.g., `confirm_pressed` / Enter / Space to start game). On "New Game":
  1. Reset `GameDirector` (score, lives, state)
  2. Set `campaign.current_index = 0`
  3. Call `enter_gameplay_scene()`
  4. Set `ActiveScene::Gameplay`
- **Draw:** Render menu UI (title text, "Press Enter to Start" prompt). No entities in the world during menu. Can use `draw_text()` or egui — implementation detail, not spec'd here.
- No gameplay systems run during `ActiveScene::Menu`.

### Entering a gameplay scene (`enter_gameplay_scene()`)

1. `world.clear()` — despawn all entities from previous scene
2. `events.drain_raw()` — clear pending events
3. `despawns.clear()` — clear despawn queue
4. Read `campaign.scenes[campaign.current_index]`
5. Spawn player via `prefabs::spawn_player()`
6. Iterate `scene_def.entities` and call the appropriate `prefabs::spawn_*` function for each `EntityDef`
7. If `scene_def.music` is `Some(id)`, emit `PlayMusic { id }`
8. Set `director.state = GameState::Playing`

### Stage cleared (all enemies dead)

Existing `StageCleared` event fires. The handler (or a new handler) checks:

- If `campaign.current_index < campaign.scenes.len() - 1`:
  - Increment `campaign.current_index`
  - Call `enter_gameplay_scene()`
  - Score and lives carry over (no reset)
- Else (final scene cleared):
  - Set `director.state = GameState::Won` (existing behavior)
  - On confirm input → return to main menu (`ActiveScene::Menu`)

### Game over (lost all lives)

Existing `PlayerDied` → `GameState::Lost` flow is unchanged. When `GameState::Lost` and player presses confirm:
- Return to `ActiveScene::Menu` (instead of calling `restart_run()`)
- `world.clear()` to clean up

---

## Changes to existing files

### `game.rs`

- Add `active_scene: ActiveScene` field to `Game`
- Add `campaign: Campaign` field to `Game` (or put it in `Resources`)
- **`update()`**: Branch on `active_scene`:
  - `Menu` → run menu update logic (check for "start game" input)
  - `Gameplay` → run existing gameplay systems (unchanged)
  - When `GameState::Won` or `GameState::Lost` and confirm pressed → transition to `Menu` instead of `restart_run()`
- **`draw()`**: Branch on `active_scene`:
  - `Menu` → draw menu
  - `Gameplay` → existing `render::draw()` + `render::draw_hud()`
- Remove `spawn_entities()` function (replaced by scene loader)
- Remove `restart_run()` (replaced by scene transitions)
- **`Game::new()`**: Load campaign + all scene defs at startup, start in `ActiveScene::Menu`

### `components.rs`

- Add `#[derive(serde::Deserialize)]` to: `EnemyKind`, `PickupKind`, `PowerupEffect`

### `events.rs`

- Add `#[derive(serde::Deserialize)]` to `MusicId`
- Add new event: `SceneTransition` (optional — could also handle transitions procedurally without events)

### `handlers.rs`

- Modify `on_stage_cleared` or add logic after it to advance campaign (this may need to happen outside the event handler since `Campaign` is not in `EventContext` — see design note below)

### `Cargo.toml`

- Add `serde` and `ron` dependencies

### `resources.rs`

- No changes to `Resources` struct if `Campaign` lives on `Game` directly
- Alternative: add `campaign: Campaign` to `Resources` and add it to `EventContext` so handlers can read campaign state

---

## Design notes

### Where does Campaign live?

**Recommended: on `Game` struct**, not in `Resources`. Reasons:
- `EventContext` already has a fixed set of fields; adding `Campaign` means changing the `EventContext` struct and all handler signatures
- Scene advancement logic (checking campaign index, calling `enter_gameplay_scene`) is orchestration, not event handling — it belongs in `Game::update()`
- The `StageCleared` handler sets `GameState::Won` as today; `Game::update()` detects `GameState::Won`, checks if there are more scenes, and either advances or stays in Won state

This means the flow is:
1. `system_process_events` runs → `on_stage_cleared` sets `director.state = GameState::Won`
2. Next `update()` tick detects `GameState::Won` → checks `campaign.has_next()` → if yes, advances to next scene and sets state back to `Playing`

This avoids Won state being visible to the player between scenes (happens within a single frame).

### Transition effect insertion point

The instant-cut flow (`world.clear()` → spawn new scene) happens inside `enter_gameplay_scene()`. To add visual transitions later, introduce an `ActiveScene::Transitioning { next_index, timer }` variant that:
1. Draws a fade-out overlay for N frames
2. Calls `enter_gameplay_scene()` at the midpoint
3. Draws a fade-in overlay
4. Switches to `ActiveScene::Gameplay`

No code for this now, but the `ActiveScene` enum makes it a clean extension.

### Background color

`SceneDef.background_color` is optional. If `Some`, call `clear_background(Color::new(r, g, b, a))` at the start of `draw()` for gameplay scenes. If `None`, use a default (current behavior).

### Error handling for RON loading

Scene/campaign loading happens at startup in `Game::new()`. On parse failure:
- `panic!` with a descriptive message (file path + ron error). This is a development-time error (bad data file), not a runtime error. No graceful fallback needed.

---

## New files

| File | Purpose |
|------|---------|
| `src/scene.rs` | `SceneDef`, `EntityDef`, `ActiveScene`, scene loading/spawning functions |
| `src/campaign.rs` | `CampaignDef`, `Campaign` struct, campaign loading |
| `assets/scenes/campaign.ron` | Campaign definition (list of scene filenames) |
| `assets/scenes/level_01.ron` | First gameplay scene (migrated from current `spawn_entities()`) |

---

## Verification

1. `cargo build` — compiles without errors
2. `cargo run` — game starts, shows main menu
3. Press Enter on menu → scene 1 loads, player spawns, enemies appear at positions from RON file
4. Destroy all enemies → next scene loads automatically (score persists, lives persist, player re-spawns)
5. Clear final scene → Won state appears, press confirm → returns to menu
6. During gameplay, lose all lives → Lost state, press confirm → returns to menu
7. Start new game from menu → score and lives are reset, campaign starts from scene 1
8. Modify a `.ron` file (change enemy position) → recompile-free change visible on next run
9. F1 debug mode still works during gameplay scenes
