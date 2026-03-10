use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
    pub rot: f32
}

pub struct BoxCollider {
    pub size: Vec2,
}