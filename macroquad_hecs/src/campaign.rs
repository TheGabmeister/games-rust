use serde::Deserialize;

use crate::scene::{load_scene_def, SceneDef};

#[derive(Deserialize)]
pub struct CampaignDef {
    pub scenes: Vec<String>,
}

pub struct Campaign {
    pub scenes: Vec<SceneDef>,
    pub current_index: usize,
}

impl Campaign {
    /// Load the campaign definition and all referenced scene files at startup.
    pub fn load(base_path: &str) -> Self {
        let campaign_path = format!("{}/campaign.ron", base_path);
        let bytes = std::fs::read(&campaign_path)
            .unwrap_or_else(|e| panic!("Failed to read campaign file '{}': {}", campaign_path, e));
        let text = std::str::from_utf8(&bytes)
            .unwrap_or_else(|e| panic!("Campaign file '{}' is not valid UTF-8: {}", campaign_path, e));
        let def: CampaignDef = ron::from_str(text)
            .unwrap_or_else(|e| panic!("Failed to parse campaign file '{}': {}", campaign_path, e));

        let scenes: Vec<SceneDef> = def
            .scenes
            .iter()
            .map(|filename| {
                let path = format!("{}/{}", base_path, filename);
                load_scene_def(&path)
            })
            .collect();

        assert!(!scenes.is_empty(), "Campaign must contain at least one scene");

        Self {
            scenes,
            current_index: 0,
        }
    }

    pub fn current_scene(&self) -> &SceneDef {
        &self.scenes[self.current_index]
    }

    pub fn has_next(&self) -> bool {
        self.current_index < self.scenes.len() - 1
    }

    pub fn advance(&mut self) {
        self.current_index += 1;
    }
}
