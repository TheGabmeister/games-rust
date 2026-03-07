use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct CameraRig {
    world_size: Vec2,
}

impl CameraRig {
    pub fn new(world_size: Vec2) -> Self {
        Self { world_size }
    }

    pub fn begin_world_pass(&self) {
        let camera = self.world_camera();
        set_camera(&camera);
    }

    pub fn begin_screen_pass(&self) {
        set_default_camera();
    }

    fn world_camera(&self) -> Camera2D {
        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.world_size.x, self.world_size.y));

        let screen_w = screen_width().max(1.0);
        let screen_h = screen_height().max(1.0);
        let scale = (screen_w / self.world_size.x).min(screen_h / self.world_size.y);

        let viewport_w = (self.world_size.x * scale).round().max(1.0) as i32;
        let viewport_h = (self.world_size.y * scale).round().max(1.0) as i32;
        let viewport_x = ((screen_w - viewport_w as f32) * 0.5).round() as i32;
        let viewport_y = ((screen_h - viewport_h as f32) * 0.5).round() as i32;
        camera.viewport = Some((viewport_x, viewport_y, viewport_w, viewport_h));

        camera
    }
}
