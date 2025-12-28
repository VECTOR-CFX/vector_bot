use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VoiceStore {
    pub channels: HashMap<u64, u64>,
}

impl VoiceStore {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("voice_channels.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("voice_channels.json", content);
        }
    }
}
