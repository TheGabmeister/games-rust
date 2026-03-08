use hecs::World;
use macroquad::prelude::*;

use crate::components::*;
use crate::resources::Resources;

/// Per-frame scratch buffer for the draw pass — allocated once and reused.
#[derive(Default)]
pub struct RenderScratch {
    drawables: Vec<(u8, Vec2, TextureId, Color)>,
}

impl RenderScratch {
    pub fn new() -> Self {
        Self {
            drawables: Vec::new(),
        }
    }
}

/// Unified sprite pass: collect all (DrawLayer, Position, Sprite) entities,
/// sort by layer, then draw. This guarantees player always renders on top of
/// enemies, and enemies on top of projectiles, regardless of spawn order.
pub fn system_draw(world: &World, res: &Resources, scratch: &mut RenderScratch) {
    scratch.drawables.clear();
    scratch.drawables.extend(
        world
            .query::<(&DrawLayer, &Position, &Sprite)>()
            .iter()
            .map(|(layer, pos, sprite)| (layer.0, pos.0, sprite.texture, sprite.tint)),
    );

    scratch.drawables.sort_unstable_by_key(|&(layer, _, _, _)| layer);

    for &(_, pos, tex_id, tint) in &scratch.drawables {
        let tex = res.texture(tex_id); // &Texture2D
        let w = tex.width();
        let h = tex.height();
        draw_texture(tex, pos.x - w / 2.0, pos.y - h / 2.0, tint);
    }
}
