use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityKind {
    Player,
    Enemy,
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Player => write!(f, "player"),
            EntityKind::Enemy  => write!(f, "enemy"),
        }
    }
}

pub struct Entity {
    pub id: u32,
    pub kind: EntityKind,
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub texture_name: Option<String>,
    pub active: bool,
}

impl Entity {
    pub fn new(id: u32, kind: EntityKind, position: Vec2, size: Vec2) -> Self {
        Self {
            id,
            kind,
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
