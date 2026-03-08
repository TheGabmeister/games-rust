use crate::transform::Transform;

pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub r:  f32,
}

pub struct CircleCollider {
    pub radius_scale: f32,
}

impl CircleCollider {
    pub fn new(radius_scale: f32) -> Self {
        Self { radius_scale }
    }

    pub fn circle(&self, t: &Transform, base_w: f32, base_h: f32) -> Circle {
        Circle {
            cx: t.x,
            cy: t.y,
            r:  base_w.min(base_h) * t.scale * self.radius_scale / 2.0,
        }
    }
}

impl Default for CircleCollider {
    fn default() -> Self {
        Self { radius_scale: 1.0 }
    }
}
