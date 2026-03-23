use hecs::World;

use crate::components::{Animator, SpriteRegion};
use crate::managers::AnimationDb;

/// Advance animation timers and update source rects for all animated entities.
pub fn system_animate(world: &mut World, anim_db: &AnimationDb, dt: f32) {
    for (animator, region) in world.query_mut::<(&mut Animator, &mut SpriteRegion)>() {
        if animator.finished {
            continue;
        }

        animator.frame_timer -= dt;

        if animator.frame_timer <= 0.0 {
            let clip = anim_db.clip(animator.sheet, animator.current_clip);

            animator.current_frame += 1;

            if animator.current_frame >= clip.frame_count {
                if clip.looping {
                    animator.current_frame = 0;
                } else {
                    animator.current_frame = clip.frame_count - 1;
                    animator.finished = true;
                }
            }

            // Accumulate remainder for frame-rate independence.
            animator.frame_timer += clip.frame_duration;
            if animator.frame_timer < 0.0 {
                animator.frame_timer = 0.0;
            }

            let abs_frame = clip.first_frame + animator.current_frame;
            region.source = anim_db.frame_rect(animator.sheet, abs_frame);
        }
    }
}
