use std::collections::HashMap;

use macroquad::prelude::Rect;

use crate::animation::{AnimClip, AnimClipName, SpriteSheetDef, SpriteSheetId};

/// Shared, immutable database of sprite sheet layouts and animation clips.
/// Built once at startup and stored in `Resources`.
pub struct AnimationDb {
    sheets: HashMap<SpriteSheetId, SpriteSheetDef>,
    clips: HashMap<(SpriteSheetId, AnimClipName), AnimClip>,
}

impl AnimationDb {
    pub fn new() -> Self {
        Self {
            sheets: HashMap::new(),
            clips: HashMap::new(),
        }
    }

    pub fn register_sheet(&mut self, id: SpriteSheetId, def: SpriteSheetDef) {
        self.sheets.insert(id, def);
    }

    pub fn register_clip(&mut self, sheet: SpriteSheetId, name: AnimClipName, clip: AnimClip) {
        self.clips.insert((sheet, name), clip);
    }

    pub fn sheet(&self, id: SpriteSheetId) -> &SpriteSheetDef {
        self.sheets
            .get(&id)
            .unwrap_or_else(|| panic!("Missing sprite sheet: {:?}", id))
    }

    pub fn clip(&self, sheet: SpriteSheetId, name: AnimClipName) -> &AnimClip {
        self.clips
            .get(&(sheet, name))
            .unwrap_or_else(|| panic!("Missing anim clip: {:?}/{:?}", sheet, name))
    }

    /// Compute the source `Rect` for a given absolute frame index within a sheet.
    pub fn frame_rect(&self, sheet: SpriteSheetId, frame_index: u16) -> Rect {
        let def = self.sheet(sheet);
        let col = frame_index % def.columns;
        let row = frame_index / def.columns;
        Rect::new(
            col as f32 * def.frame_width as f32,
            row as f32 * def.frame_height as f32,
            def.frame_width as f32,
            def.frame_height as f32,
        )
    }
}
