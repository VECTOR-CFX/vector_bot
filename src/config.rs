use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub roles: Roles,
    pub channels: Channels,
    pub categories: Categories,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Roles {
    pub staff_role_id: u64,
    pub client_role_id: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Channels {
    pub log_channel_id: u64,
    pub jtc_channel_ids: Vec<u64>,
    pub voice_log_channel_id: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Categories {
    pub partnership: u64,
    pub recruitment: u64,
    pub support: u64,
    pub other: u64,
    pub voice_category_id: u64,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
