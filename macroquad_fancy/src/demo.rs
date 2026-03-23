use macroquad::prelude::*;

use crate::postfx::PostFxPipeline;
use crate::scene::Scene;
use crate::scenes::aurora::AuroraScene;
use crate::scenes::fire::FireScene;
use crate::scenes::moire::MoireScene;
use crate::scenes::spirograph::SpirographScene;
use crate::scenes::starfield::StarfieldScene;
use crate::scenes::voronoi::VoronoiScene;

pub struct DemoRunner {
    scenes: Vec<Box<dyn Scene>>,
    current_scene: usize,
    scene_time: f32,
    postfx: PostFxPipeline,
    rt_current: RenderTarget,
    total_time: f32,
}

impl DemoRunner {
    pub fn new() -> Self {
        let w = screen_width() as u32;
        let h = screen_height() as u32;

        let rt_current = render_target(w, h);
        rt_current.texture.set_filter(FilterMode::Linear);

        let mut scenes: Vec<Box<dyn Scene>> = vec![
            Box::new(StarfieldScene::new()),
            Box::new(FireScene::new()),
            Box::new(SpirographScene::new()),
            Box::new(MoireScene::new()),
            Box::new(AuroraScene::new()),
            Box::new(VoronoiScene::new()),
        ];

        scenes[0].init();

        Self {
            scenes,
            current_scene: 0,
            scene_time: 0.0,
            postfx: PostFxPipeline::new(w, h),
            rt_current,
            total_time: 0.0,
        }
    }

    fn switch_to_scene(&mut self, idx: usize) {
        if idx != self.current_scene {
            self.current_scene = idx;
            self.scene_time = 0.0;
            self.scenes[idx].init();
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.total_time += dt;
        self.scene_time += dt;
        self.scenes[self.current_scene].update(self.scene_time, dt);

        // Handle input
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_position();
            self.handle_click(mouse.0, mouse.1);
        }
    }

    fn handle_click(&mut self, mx: f32, my: f32) {
        let sw = screen_width();
        let sh = screen_height();

        // Scene navigation buttons (bottom-center)
        let nav_y = sh - 50.0;
        let prev_x = sw / 2.0 - 160.0;
        let next_x = sw / 2.0 + 60.0;
        let btn_w = 100.0;
        let btn_h = 36.0;

        if hit_test(mx, my, prev_x, nav_y, btn_w, btn_h) {
            let count = self.scenes.len();
            let prev = (self.current_scene + count - 1) % count;
            self.switch_to_scene(prev);
            return;
        }
        if hit_test(mx, my, next_x, nav_y, btn_w, btn_h) {
            let next = (self.current_scene + 1) % self.scenes.len();
            self.switch_to_scene(next);
            return;
        }

        // Post-FX toggle buttons (top-right, vertical stack)
        let toggle_x = sw - 160.0;
        let toggle_w = 140.0;
        let toggle_h = 30.0;
        let toggle_start_y = 50.0;
        let toggle_spacing = 38.0;

        let toggles_clicked = [
            hit_test(mx, my, toggle_x, toggle_start_y, toggle_w, toggle_h),
            hit_test(mx, my, toggle_x, toggle_start_y + toggle_spacing, toggle_w, toggle_h),
            hit_test(mx, my, toggle_x, toggle_start_y + toggle_spacing * 2.0, toggle_w, toggle_h),
            hit_test(mx, my, toggle_x, toggle_start_y + toggle_spacing * 3.0, toggle_w, toggle_h),
            hit_test(mx, my, toggle_x, toggle_start_y + toggle_spacing * 4.0, toggle_w, toggle_h),
        ];

        if toggles_clicked[0] { self.postfx.bloom_enabled = !self.postfx.bloom_enabled; }
        if toggles_clicked[1] { self.postfx.chromatic_enabled = !self.postfx.chromatic_enabled; }
        if toggles_clicked[2] { self.postfx.wave_enabled = !self.postfx.wave_enabled; }
        if toggles_clicked[3] { self.postfx.crt_enabled = !self.postfx.crt_enabled; }
        if toggles_clicked[4] { self.postfx.vignette_enabled = !self.postfx.vignette_enabled; }
    }

    pub fn draw(&self) {
        let w = screen_width();
        let h = screen_height();

        // Draw current scene to render target
        let cam = Camera2D {
            render_target: Some(self.rt_current.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h))
        };
        set_camera(&cam);
        self.scenes[self.current_scene].draw();

        // Apply post-processing chain (ends by drawing to screen)
        self.postfx.apply(&self.rt_current, self.total_time);

        // Draw UI overlay on top
        self.draw_overlay();
    }

    fn draw_overlay(&self) {
        let sw = screen_width();
        let sh = screen_height();

        // FPS counter (top-left)
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            24.0,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.6),
        );

        // Scene name (bottom-center, between nav buttons)
        let name = self.scenes[self.current_scene].name();
        let label = format!("{}/{} {}", self.current_scene + 1, self.scenes.len(), name);
        let dims = measure_text(&label, None, 24, 1.0);
        draw_text(
            &label,
            sw / 2.0 - dims.width / 2.0,
            sh - 58.0,
            24.0,
            Color::new(1.0, 1.0, 1.0, 0.9),
        );

        // Navigation buttons
        let nav_y = sh - 50.0;
        let prev_x = sw / 2.0 - 160.0;
        let next_x = sw / 2.0 + 60.0;
        draw_button(prev_x, nav_y, 100.0, 36.0, "< Prev", None);
        draw_button(next_x, nav_y, 100.0, 36.0, "Next >", None);

        // Post-FX toggle buttons (top-right)
        let toggle_x = sw - 160.0;
        let toggle_w = 140.0;
        let toggle_h = 30.0;
        let toggle_start_y = 50.0;
        let toggle_spacing = 38.0;

        let effects: [(&str, bool); 5] = [
            ("Bloom", self.postfx.bloom_enabled),
            ("Chromatic", self.postfx.chromatic_enabled),
            ("Wave", self.postfx.wave_enabled),
            ("CRT", self.postfx.crt_enabled),
            ("Vignette", self.postfx.vignette_enabled),
        ];

        // Label
        draw_text(
            "Post-FX",
            toggle_x,
            toggle_start_y - 10.0,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.6),
        );

        for (i, (name, enabled)) in effects.iter().enumerate() {
            let y = toggle_start_y + toggle_spacing * i as f32;
            draw_button(toggle_x, y, toggle_w, toggle_h, name, Some(*enabled));
        }
    }
}

fn hit_test(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn draw_button(x: f32, y: f32, w: f32, h: f32, label: &str, active: Option<bool>) {
    let mouse = mouse_position();
    let hovered = hit_test(mouse.0, mouse.1, x, y, w, h);

    let (bg, text_color) = match active {
        None => {
            // Action button (prev/next)
            if hovered {
                (Color::new(0.4, 0.4, 0.5, 0.85), WHITE)
            } else {
                (Color::new(0.2, 0.2, 0.3, 0.75), Color::new(0.9, 0.9, 0.9, 0.9))
            }
        }
        Some(true) => {
            // Toggle ON
            if hovered {
                (Color::new(0.1, 0.6, 0.2, 0.9), WHITE)
            } else {
                (Color::new(0.1, 0.5, 0.15, 0.8), Color::new(0.9, 1.0, 0.9, 0.95))
            }
        }
        Some(false) => {
            // Toggle OFF
            if hovered {
                (Color::new(0.4, 0.15, 0.15, 0.85), Color::new(0.8, 0.8, 0.8, 0.8))
            } else {
                (Color::new(0.25, 0.1, 0.1, 0.7), Color::new(0.5, 0.5, 0.5, 0.6))
            }
        }
    };

    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(1.0, 1.0, 1.0, 0.3));

    let dims = measure_text(label, None, 18, 1.0);
    draw_text(
        label,
        x + (w - dims.width) / 2.0,
        y + (h + dims.height) / 2.0 - 2.0,
        18.0,
        text_color,
    );
}
