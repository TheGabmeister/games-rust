use hecs::World;

use crate::components::{AnimDemo, Animator, SpriteRegion};
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

/// Cycle through a list of animation clips on a timer (demo/testing).
pub fn system_anim_demo(world: &mut World, anim_db: &AnimationDb, dt: f32) {
    for (demo, animator) in world.query_mut::<(&mut AnimDemo, &mut Animator)>() {
        demo.timer -= dt;
        if demo.timer <= 0.0 {
            demo.current_index = (demo.current_index + 1) % demo.clips.len();
            animator.play(demo.clips[demo.current_index], anim_db);
            demo.timer = demo.interval;
        }
    }
}
