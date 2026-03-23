pub trait Scene {
    /// Called when the scene becomes active (entering).
    fn init(&mut self);

    /// Called every frame. `t` is seconds since this scene became active, `dt` is frame delta.
    fn update(&mut self, t: f32, dt: f32);

    /// Draw scene content to the currently set render target.
    fn draw(&self);

    /// Human-readable name for the overlay.
    fn name(&self) -> &str;
}
