use crate::managers::AnimationDb;

/// Build the animation database with all sprite sheet definitions and clips.
/// Add your Aseprite-exported sheets and clips here.
pub fn build_animation_db() -> AnimationDb {
    let db = AnimationDb::new();

    // Example: uncomment and adapt when you have a real sprite sheet.
    //
    // db.register_sheet(SpriteSheetId::PlayerLink, SpriteSheetDef {
    //     texture: TextureId::LinkSheet,
    //     frame_width: 16,
    //     frame_height: 16,
    //     columns: 8,
    // });
    //
    // db.register_clip(SpriteSheetId::PlayerLink, AnimClipName::WalkDown, AnimClip {
    //     first_frame: 0,
    //     frame_count: 4,
    //     frame_duration: 0.12,
    //     looping: true,
    // });

    db
}
