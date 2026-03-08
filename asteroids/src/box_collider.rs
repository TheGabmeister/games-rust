use macroquad::prelude::*;

use crate::transform::Transform;

pub struct Obb {
    pub cx:  f32,
    pub cy:  f32,
    pub hw:  f32,  // half-width (unrotated)
    pub hh:  f32,  // half-height (unrotated)
    pub rot: f32,  // radians
}

impl Obb {
    pub fn corners(&self) -> [Vec2; 4] {
        let cos_r = self.rot.cos();
        let sin_r = self.rot.sin();
        [(-self.hw, -self.hh), (self.hw, -self.hh), (self.hw, self.hh), (-self.hw, self.hh)]
            .map(|(cx, cy)| vec2(
                self.cx + cx * cos_r - cy * sin_r,
                self.cy + cx * sin_r + cy * cos_r,
            ))
    }

    pub fn overlaps(&self, other: &Obb) -> bool {
        let a = self.corners();
        let b = other.corners();
        let axes = [
            vec2( self.rot.cos(),  self.rot.sin()),
            vec2(-self.rot.sin(),  self.rot.cos()),
            vec2( other.rot.cos(), other.rot.sin()),
            vec2(-other.rot.sin(), other.rot.cos()),
        ];
        for axis in axes {
            let (a_min, a_max) = project(&a, axis);
            let (b_min, b_max) = project(&b, axis);
            if a_max < b_min || b_max < a_min { return false; }
        }
        true
    }
}

fn project(corners: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for &c in corners {
        let p = c.dot(axis);
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

pub struct BoxCollider {
    pub x_scale: f32,
    pub y_scale: f32,
}

impl BoxCollider {
    pub fn new(x_scale: f32, y_scale: f32) -> Self {
        Self { x_scale, y_scale }
    }

    pub fn obb(&self, t: &Transform, base_w: f32, base_h: f32) -> Obb {
        Obb {
            cx:  t.x,
            cy:  t.y,
            hw:  base_w * t.scale * self.x_scale / 2.0,
            hh:  base_h * t.scale * self.y_scale / 2.0,
            rot: t.rot,
        }
    }
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self { x_scale: 1.0, y_scale: 1.0 }
    }
}
