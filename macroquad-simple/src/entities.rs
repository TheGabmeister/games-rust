use macroquad::prelude::*;

pub struct Entity {
    pub id: u32,
    pub name: String,
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub texture_name: Option<String>,
    pub active: bool,
}

impl Entity {
    pub fn new(id: u32, name: &str, position: Vec2, size: Vec2) -> Self {
        Self {
            id,
            name: name.to_string(),
            position,
            size,
            velocity: Vec2::ZERO,
            texture_name: None,
            active: true,
        }
    }

    pub fn with_texture(mut self, name: &str) -> Self {
        self.texture_name = Some(name.to_string());
        self
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }
}
