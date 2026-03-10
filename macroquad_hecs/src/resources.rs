pub struct Resources {
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            
        }
    }
}

/// Snapshot of player/input intent captured once per frame.
#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub confirm_pressed: bool,
    pub cancel_pressed: bool,
    pub debug_toggle_pressed: bool,
}

impl InputState {

}