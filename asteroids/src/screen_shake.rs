use macroquad::rand::gen_range;

const MAX_OFFSET: f32 = 8.0;
const DECAY_RATE: f32 = 2.0;

pub struct ScreenShake {
    trauma: f32,
    offset_x: f32,
    offset_y: f32,
}

impl ScreenShake {
    pub fn new() -> Self {
        Self { trauma: 0.0, offset_x: 0.0, offset_y: 0.0 }
    }

    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    pub fn update(&mut self, dt: f32) {
        self.trauma = (self.trauma - DECAY_RATE * dt).max(0.0);
        if self.trauma > 0.0 {
            let intensity = self.trauma * self.trauma;
            self.offset_x = intensity * MAX_OFFSET * gen_range(-1.0f32, 1.0);
            self.offset_y = intensity * MAX_OFFSET * gen_range(-1.0f32, 1.0);
        } else {
            self.offset_x = 0.0;
            self.offset_y = 0.0;
        }
    }

    pub fn offset(&self) -> (f32, f32) {
        (self.offset_x, self.offset_y)
    }
}
