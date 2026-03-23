use macroquad::prelude::*;

use crate::animation::{AnimClipName, SpriteSheetId};
use crate::managers::AnimationDb;

// ---------------------------------------------------------------------------
// Spatial
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
    pub rot: f32, // radians, 0 = up
}

impl Transform {
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            rot: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    pub linear: Vec2, // pixels/sec
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { linear: vec2(x, y) }
    }
    pub fn zero() -> Self {
        Self { linear: Vec2::ZERO }
    }
}

// ---------------------------------------------------------------------------
// Collision
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box centered on Transform.pos (half-extents).
#[derive(Clone, Copy, Debug)]
pub struct BoxCollider {
    pub half: Vec2,
}

impl BoxCollider {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            half: vec2(w * 0.5, h * 0.5),
        }
    }
}

/// Circle collider centered on Transform.pos.
#[derive(Clone, Copy, Debug)]
pub struct CircleCollider {
    pub radius: f32,
}

impl CircleCollider {
    pub fn new(r: f32) -> Self {
        Self { radius: r }
    }
}

/// Bitmask controlling which collision layers an entity belongs to and checks against.
#[derive(Clone, Copy, Debug)]
pub struct CollisionLayer {
    pub member: u32, // layers this entity is ON
    pub mask: u32,   // layers this entity checks against
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureId {
    PlayerShip,
    PlayerShipLeft,
    PlayerShipRight,
    PlayerLaser,
    EnemyShipBlack,
    EnemyShipBlue,
    EnemyShipGreen,
    EnemyShipRed,
    EnemyLaser,
    PickupLife,
    PickupStar,
    PowerupBolt,
    PowerupShield,
    OldHero,
}

pub struct Sprite {
    pub texture: TextureId,
    pub tint: Color,
}

impl Sprite {
    pub fn new(texture: TextureId) -> Self {
        Self {
            texture,
            tint: WHITE,
        }
    }
}

/// Draw ordering: lower = rendered first = behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawLayer(pub u8);

/// Per-entity animation playback state. Pair with `SpriteRegion` and a `Sprite`
/// whose `texture` points to the sprite sheet atlas.
pub struct Animator {
    pub sheet: SpriteSheetId,
    pub current_clip: AnimClipName,
    pub current_frame: u16,
    pub frame_timer: f32,
    pub finished: bool,
}

impl Animator {
    pub fn new(sheet: SpriteSheetId, clip: AnimClipName, anim_db: &AnimationDb) -> Self {
        let c = anim_db.clip(sheet, clip);
        Self {
            sheet,
            current_clip: clip,
            current_frame: 0,
            frame_timer: c.frame_duration,
            finished: false,
        }
    }

    /// Switch to a different clip. No-op if already playing that clip.
    pub fn play(&mut self, clip: AnimClipName, anim_db: &AnimationDb) {
        if self.current_clip == clip && !self.finished {
            return;
        }
        let c = anim_db.clip(self.sheet, clip);
        self.current_clip = clip;
        self.current_frame = 0;
        self.frame_timer = c.frame_duration;
        self.finished = false;
    }
}

/// Source rectangle within a sprite sheet texture. Updated each tick by
/// `system_animate`. If absent on an entity, the render system draws the
/// full texture (backward compatible with static sprites).
pub struct SpriteRegion {
    pub source: Rect,
    #[allow(dead_code)]
    pub size: Vec2,
}

impl SpriteRegion {
    pub fn new(sheet: SpriteSheetId, clip: AnimClipName, anim_db: &AnimationDb) -> Self {
        let c = anim_db.clip(sheet, clip);
        let rect = anim_db.frame_rect(sheet, c.first_frame);
        let def = anim_db.sheet(sheet);
        Self {
            source: rect,
            size: vec2(def.frame_width as f32, def.frame_height as f32),
        }
    }
}

/// Demo component that cycles an entity through a list of animation clips.
pub struct AnimDemo {
    pub clips: &'static [AnimClipName],
    pub current_index: usize,
    pub timer: f32,
    pub interval: f32,
}

// ---------------------------------------------------------------------------
// Entity tags
// ---------------------------------------------------------------------------

pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}

pub struct Projectile {
    pub owner: ProjectileOwner,
}

// ---------------------------------------------------------------------------
// Gameplay
// ---------------------------------------------------------------------------

/// Entity is despawned when remaining reaches 0.
#[derive(Clone, Copy, Debug)]
pub struct Lifetime {
    pub remaining: f32,
}

impl Lifetime {
    pub fn new(secs: f32) -> Self {
        Self { remaining: secs }
    }
}

/// Per-entity fire cooldown timer.
#[derive(Clone, Copy, Debug)]
pub struct FireTimer {
    pub cooldown: f32, // seconds between shots
    pub timer: f32,    // counts down; fires when <= 0
}

impl FireTimer {
    pub fn new(cooldown: f32) -> Self {
        Self {
            cooldown,
            timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Black,
    Blue,
    Green,
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Life,
    Star,
}

pub struct Pickup {
    pub kind: PickupKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerupEffect {
    Bolt,   // rapid fire
    Shield, // damage immunity
}

pub struct PowerupPickup {
    pub effect: PowerupEffect,
    pub duration: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ActivePowerups {
    pub bolt_remaining: f32,
    pub shield_remaining: f32,
}

/// Score awarded when this entity is destroyed.
#[derive(Clone, Copy, Debug)]
pub struct ScoreValue(pub u32);
