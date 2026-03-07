pub mod audio;
pub mod collision;
pub mod input;
pub mod movement;
pub mod ui;

#[derive(Clone, Debug)]
pub struct FixedTimestepScheduler {
    fixed_dt: f32,
    max_frame_dt: f32,
    max_steps_per_frame: u32,
    accumulator: f32,
}

impl FixedTimestepScheduler {
    pub fn new(fixed_dt: f32, max_frame_dt: f32, max_steps_per_frame: u32) -> Self {
        assert!(fixed_dt > 0.0, "fixed_dt must be positive");
        assert!(max_frame_dt > 0.0, "max_frame_dt must be positive");
        assert!(max_steps_per_frame > 0, "max_steps_per_frame must be > 0");

        Self {
            fixed_dt,
            max_frame_dt,
            max_steps_per_frame,
            accumulator: 0.0,
        }
    }

    pub fn begin_frame(&mut self, frame_dt: f32) -> u32 {
        self.accumulator += frame_dt.clamp(0.0, self.max_frame_dt);

        let mut steps = 0;
        while self.accumulator >= self.fixed_dt && steps < self.max_steps_per_frame {
            self.accumulator -= self.fixed_dt;
            steps += 1;
        }

        // Avoid spiraling updates when frame stalls for too long.
        if steps == self.max_steps_per_frame {
            self.accumulator = 0.0;
        }

        steps
    }

    pub const fn fixed_dt(&self) -> f32 {
        self.fixed_dt
    }

    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.fixed_dt).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedTimestepScheduler;

    #[test]
    fn scheduler_steps_and_alpha_are_stable() {
        let mut scheduler = FixedTimestepScheduler::new(1.0, 10.0, 8);

        assert_eq!(scheduler.begin_frame(2.4), 2);
        assert!((scheduler.alpha() - 0.4).abs() < f32::EPSILON);

        assert_eq!(scheduler.begin_frame(0.2), 0);
        assert!((scheduler.alpha() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn scheduler_caps_backlog() {
        let mut scheduler = FixedTimestepScheduler::new(1.0, 100.0, 3);

        assert_eq!(scheduler.begin_frame(10.0), 3);
        assert_eq!(scheduler.alpha(), 0.0);
    }
}
