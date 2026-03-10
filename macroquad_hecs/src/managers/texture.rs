use std::collections::HashMap;
use macroquad::prelude::Texture2D;
use crate::components::TextureId;
use super::TextureManager;

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
