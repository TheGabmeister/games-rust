use macroquad::prelude::{Color, Vec2};

// ── Position / Physics ────────────────────────────────────────────────────────

/// Pixel position of the entity's top-left corner.
pub struct Position(pub Vec2);

/// Horizontal velocity in pixels per second.
pub struct Velocity(pub f32);

/// Width × height in pixels (axis-aligned bounding box).
pub struct Size(pub Vec2);

/// Primary draw color for this entity.
pub struct DrawColor(pub Color);

// ── Frog ──────────────────────────────────────────────────────────────────────

/// Marker: this entity is the player frog.
pub struct Frog;

/// Grid-snapped cell the frog currently occupies (updated on hop).
pub struct FrogCell {
    pub col: i32,
    pub row: i32,
}

/// Best (lowest-numbered) row reached this life. Used to award hop points
/// only for genuine forward progress.
pub struct BestRow(pub i32);

/// Smooth hop animation lerp.  Removed when complete.
pub struct HopAnim {
    pub t: f32,
    pub from: Vec2,
    pub to: Vec2,
}

/// Frog is currently riding this platform entity.
pub struct RidingPlatform(pub hecs::Entity);

// ── Road entities ─────────────────────────────────────────────────────────────

/// Marker: this entity is a road vehicle (kills the frog on contact).
pub struct Vehicle;

// ── River entities ────────────────────────────────────────────────────────────

/// Marker: this entity is a river platform the frog can ride.
pub struct Platform;

/// Turtle diving state machine.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DivePhase {
    Surface,   // fully visible, rideable
    Diving,    // sinking, still rideable briefly (use alpha fade)
    Submerged, // invisible, NOT rideable
    Rising,    // re-emerging, not rideable yet
}

pub struct TurtleDive {
    pub phase: DivePhase,
    pub timer: f32, // seconds until next phase transition
}

impl TurtleDive {
    /// Duration of each phase in seconds.
    pub fn phase_duration(phase: DivePhase) -> f32 {
        match phase {
            DivePhase::Surface   => 8.0,
            DivePhase::Diving    => 1.5,
            DivePhase::Submerged => 3.0,
            DivePhase::Rising    => 1.5,
        }
    }

    pub fn next_phase(phase: DivePhase) -> DivePhase {
        match phase {
            DivePhase::Surface   => DivePhase::Diving,
            DivePhase::Diving    => DivePhase::Submerged,
            DivePhase::Submerged => DivePhase::Rising,
            DivePhase::Rising    => DivePhase::Surface,
        }
    }

    /// Returns true if the frog can stand on this turtle right now.
    pub fn is_rideable(&self) -> bool {
        matches!(self.phase, DivePhase::Surface | DivePhase::Diving)
    }
}

/// Wrapping behaviour: when the entity leaves [x_min, x_max] it reappears
/// on the opposite side.
pub struct WrapBounds {
    pub x_min: f32,
    pub x_max: f32,
}

// ── Home pockets ──────────────────────────────────────────────────────────────

/// One of the five frog-home pockets at the top of the screen.
pub struct Home {
    pub idx: usize,  // 0..4 (index into HOME_COLS)
    pub filled: bool,
}

// ── Bonus fly ─────────────────────────────────────────────────────────────────

/// A bonus fly sitting inside a home pocket.
pub struct Fly {
    pub home_idx: usize,
    pub timer: f32, // seconds remaining before despawn
}

// ── Meta / game-state singletons ─────────────────────────────────────────────
// These all live on a single "meta" entity spawned once per game.

pub struct Lives(pub i32);
pub struct Score(pub i32);
pub struct Level(pub i32);
pub struct LevelTimer(pub f32);
pub struct HomesProgress(pub [bool; 5]);

// ── Per-frog transient state ──────────────────────────────────────────────────

/// Death animation in progress; value = seconds remaining.
pub struct DeathAnim(pub f32);

/// Brief input-blocking delay after re-spawning; value = seconds remaining.
pub struct RespawnDelay(pub f32);
