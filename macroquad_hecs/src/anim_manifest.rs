use crate::animation::{AnimClip, AnimClipName, SpriteSheetDef, SpriteSheetId};
use crate::components::TextureId;
use crate::managers::AnimationDb;

/// Build the animation database with all sprite sheet definitions and clips.
pub fn build_animation_db() -> AnimationDb {
    let mut db = AnimationDb::new();

    // old_hero.png: 64x85, 16x17 frames, 4 columns, 5 rows
    db.register_sheet(
        SpriteSheetId::OldHero,
        SpriteSheetDef {
            texture: TextureId::OldHero,
            frame_width: 16,
            frame_height: 17,
            columns: 4,
        },
    );

    // Row 0: Idle (frames 0–3)
    db.register_clip(
        SpriteSheetId::OldHero,
        AnimClipName::Idle,
        AnimClip {
            first_frame: 0,
            frame_count: 4,
            frame_duration: 0.15,
            looping: true,
        },
    );

    // Row 1: Walk (frames 4–7)
    db.register_clip(
        SpriteSheetId::OldHero,
        AnimClipName::Walk,
        AnimClip {
            first_frame: 4,
            frame_count: 4,
            frame_duration: 0.15,
            looping: true,
        },
    );

    // Row 2: Run (frames 8–11)
    db.register_clip(
        SpriteSheetId::OldHero,
        AnimClipName::Run,
        AnimClip {
            first_frame: 8,
            frame_count: 4,
            frame_duration: 0.15,
            looping: true,
        },
    );

    // Row 3: Jump (frames 12–15)
    db.register_clip(
        SpriteSheetId::OldHero,
        AnimClipName::Jump,
        AnimClip {
            first_frame: 12,
            frame_count: 4,
            frame_duration: 0.15,
            looping: true,
        },
    );

    // Row 4: Fight (frames 16–18, only 3 frames)
    db.register_clip(
        SpriteSheetId::OldHero,
        AnimClipName::Fight,
        AnimClip {
            first_frame: 16,
            frame_count: 3,
            frame_duration: 0.15,
            looping: true,
        },
    );

    db
}
