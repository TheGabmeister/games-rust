#![allow(dead_code)]

use crate::components::TextureId;

// ---------------------------------------------------------------------------
// Sprite sheet identity
// ---------------------------------------------------------------------------

/// Identifies a sprite sheet definition in the AnimationDb.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpriteSheetId {
    // Add entries here as you create sprite sheets, e.g.:
    // PlayerLink,
    // Npc,
}

/// Named animation clips within a sprite sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimClipName {
    Idle,
    WalkUp,
    WalkDown,
    WalkLeft,
    WalkRight,
    AttackUp,
    AttackDown,
    AttackLeft,
    AttackRight,
}

// ---------------------------------------------------------------------------
// Definitions (shared, immutable after setup)
// ---------------------------------------------------------------------------

/// Describes the grid layout of a uniform sprite sheet (e.g. an Aseprite export).
pub struct SpriteSheetDef {
    /// Which loaded texture this sheet lives in.
    pub texture: TextureId,
    /// Width of a single frame in pixels.
    pub frame_width: u16,
    /// Height of a single frame in pixels.
    pub frame_height: u16,
    /// Number of frames per row in the sheet.
    pub columns: u16,
}

/// A single animation clip within a sprite sheet.
pub struct AnimClip {
    /// Index of the first frame in the sheet's grid (0-based, left-to-right top-to-bottom).
    pub first_frame: u16,
    /// Number of frames in this clip.
    pub frame_count: u16,
    /// Seconds per frame (e.g. 0.1 = 10 fps).
    pub frame_duration: f32,
    /// Whether the clip loops or holds on the last frame.
    pub looping: bool,
}
