use macroquad::prelude::*;
use hecs::World;

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
        let w   = tex.width();
        let h   = tex.height();
        draw_texture(tex, pos.x - w / 2.0, pos.y - h / 2.0, tint);
    }

    // HP / kill-count labels — drawn on top of all sprites.
    for (pos, hp, kc) in world.query::<(&Position, &Health, &KillCount)>().iter() {
        draw_text(
            &format!("HP:{} K:{}", hp.0, kc.0),
            pos.0.x - 10.0,
            pos.0.y - tex_label_offset(res),
            14.0,
            WHITE,
        );
    }
}

fn tex_label_offset(res: &Resources) -> f32 {
    res.texture(TextureId::EnemyBlack).height() / 2.0 + 6.0
}
