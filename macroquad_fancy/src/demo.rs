use macroquad::prelude::*;

use crate::postfx::PostFxPipeline;
use crate::scene::Scene;
use crate::scenes::aurora::AuroraScene;
use crate::scenes::fire::FireScene;
use crate::scenes::moire::MoireScene;
use crate::scenes::spirograph::SpirographScene;
use crate::scenes::starfield::StarfieldScene;
use crate::scenes::voronoi::VoronoiScene;
use crate::transitions::TransitionSystem;

const SCENE_DURATION: f32 = 10.0;
const TRANSITION_DURATION: f32 = 1.5;

enum RunnerState {
    Playing,
    TransitionOut { next_scene: usize },
}

pub struct DemoRunner {
    scenes: Vec<Box<dyn Scene>>,
    current_scene: usize,
    scene_time: f32,
    state: RunnerState,
    transition_time: f32,
    transition_idx: usize,
    postfx: PostFxPipeline,
    transitions: TransitionSystem,
    rt_current: RenderTarget,
    rt_next: RenderTarget,
    rt_composite: RenderTarget,
    total_time: f32,
    overlay_alpha: f32,
}

impl DemoRunner {
    pub fn new() -> Self {
        let w = screen_width() as u32;
        let h = screen_height() as u32;

        let rt_current = render_target(w, h);
        rt_current.texture.set_filter(FilterMode::Linear);
        let rt_next = render_target(w, h);
        rt_next.texture.set_filter(FilterMode::Linear);
        let rt_composite = render_target(w, h);
        rt_composite.texture.set_filter(FilterMode::Linear);

        let mut scenes: Vec<Box<dyn Scene>> = vec![
            Box::new(StarfieldScene::new()),
            Box::new(FireScene::new()),
            Box::new(SpirographScene::new()),
            Box::new(MoireScene::new()),
            Box::new(AuroraScene::new()),
            Box::new(VoronoiScene::new()),
        ];

        // Initialize the first scene
        scenes[0].init();

        Self {
            scenes,
            current_scene: 0,
            scene_time: 0.0,
            state: RunnerState::Playing,
            transition_time: 0.0,
            transition_idx: 0,
            postfx: PostFxPipeline::new(w, h),
            transitions: TransitionSystem::new(),
            rt_current,
            rt_next,
            rt_composite,
            total_time: 0.0,
            overlay_alpha: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.total_time += dt;
        self.scene_time += dt;

        match self.state {
            RunnerState::Playing => {
                self.scenes[self.current_scene].update(self.scene_time, dt);

                // Fade overlay alpha
                if self.scene_time < 2.0 {
                    self.overlay_alpha = 1.0 - (self.scene_time / 2.0);
                } else {
                    self.overlay_alpha = 0.0;
                }

                if self.scene_time >= SCENE_DURATION {
                    let next = (self.current_scene + 1) % self.scenes.len();
                    self.scenes[next].init();
                    self.state = RunnerState::TransitionOut { next_scene: next };
                    self.transition_time = 0.0;
                    self.overlay_alpha = 0.0;
                }
            }
            RunnerState::TransitionOut { next_scene } => {
                self.transition_time += dt;
                self.scenes[self.current_scene].update(self.scene_time, dt);
                self.scenes[next_scene].update(self.transition_time, dt);

                if self.transition_time >= TRANSITION_DURATION {
                    self.current_scene = next_scene;
                    self.scene_time = self.transition_time;
                    self.transition_time = 0.0;
                    self.transition_idx += 1;
                    self.state = RunnerState::Playing;
                    self.overlay_alpha = 1.0;
                }
            }
        }
    }

    pub fn draw(&self) {
        let w = screen_width();
        let h = screen_height();

        // Draw current scene to rt_current
        let cam_current = Camera2D {
            render_target: Some(self.rt_current.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h))
        };
        set_camera(&cam_current);
        self.scenes[self.current_scene].draw();

        // Determine the source for post-processing
        let postfx_source = match self.state {
            RunnerState::Playing => &self.rt_current,
            RunnerState::TransitionOut { next_scene } => {
                // Draw next scene to rt_next
                let cam_next = Camera2D {
                    render_target: Some(self.rt_next.clone()),
                    ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h))
                };
                set_camera(&cam_next);
                self.scenes[next_scene].draw();

                // Composite via transition
                let progress = self.transition_time / TRANSITION_DURATION;
                self.transitions.apply(
                    &self.rt_current,
                    &self.rt_next,
                    &self.rt_composite,
                    self.transition_idx,
                    progress,
                );

                &self.rt_composite
            }
        };

        // Apply post-processing chain (ends by drawing to screen)
        self.postfx.apply(postfx_source, self.total_time);

        // Draw overlay on top (already on screen from postfx)
        self.draw_overlay();
    }

    fn draw_overlay(&self) {
        let scene_name = match self.state {
            RunnerState::Playing => self.scenes[self.current_scene].name(),
            RunnerState::TransitionOut { next_scene } => self.scenes[next_scene].name(),
        };

        // Scene name with fade
        if self.overlay_alpha > 0.01 {
            let alpha = self.overlay_alpha.clamp(0.0, 1.0);
            draw_text(
                scene_name,
                screen_width() / 2.0 - measure_text(scene_name, None, 36, 1.0).width / 2.0,
                screen_height() / 2.0,
                36.0,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }

        // FPS counter
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            24.0,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.6),
        );
    }
}
