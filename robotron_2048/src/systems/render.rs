use hecs::World;
use macroquad::prelude::*;

use crate::components::*;
use crate::resources::Resources;

/// Unified sprite pass: collect all (DrawLayer, Position, Sprite) entities,
/// sort by layer, then draw. This guarantees player always renders on top of
/// enemies, and enemies on top of projectiles, regardless of spawn order.
pub fn system_draw(world: &World, res: &Resources) {
    // Collect owned data so we can sort without holding a QueryBorrow.
    let mut drawables: Vec<(u8, Vec2, TextureId, Color)> = world
        .query::<(&DrawLayer, &Position, &Sprite)>()
        .iter()
        .map(|(layer, pos, sprite)| (layer.0, pos.0, sprite.texture, sprite.tint))
        .collect();

    drawables.sort_unstable_by_key(|&(layer, _, _, _)| layer);

    for (_, pos, tex_id, tint) in drawables {
        let tex = res.texture(tex_id); // &Texture2D
        let w = tex.width();
        let h = tex.height();
        draw_texture(tex, pos.x - w / 2.0, pos.y - h / 2.0, tint);
    }
}
