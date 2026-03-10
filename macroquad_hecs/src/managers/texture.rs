use crate::components::TextureId;
use macroquad::prelude::Texture2D;
use std::collections::HashMap;

pub struct TextureManager {
    textures: HashMap<TextureId, Texture2D>,
}

impl TextureManager {
    pub fn new(textures: HashMap<TextureId, Texture2D>) -> Self {
        Self { textures }
    }

    pub fn texture(&self, id: TextureId) -> &Texture2D {
        self.textures
            .get(&id)
            .unwrap_or_else(|| panic!("Missing texture for id: {:?}", id))
    }
}
