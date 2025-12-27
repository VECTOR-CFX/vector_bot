use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TicketCategory {
    Partnership,
    Recruitment,
    Support,
    Other,
}

impl TicketCategory {
    pub fn to_string(&self) -> String {
        match self {
            TicketCategory::Partnership => "Partenariat".to_string(),
            TicketCategory::Recruitment => "Recrutement".to_string(),
            TicketCategory::Support => "Support".to_string(),
            TicketCategory::Other => "Autres".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TicketState {
    ChoosingLanguage,
    ChoosingCategory { language: String },
    WritingMessage { language: String, category: TicketCategory },
    InTicket { channel_id: serenity::ChannelId },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketInfo {
    pub user_id: u64,
    pub channel_id: u64,
    pub category: TicketCategory,
    pub created_at: i64, 
    pub initial_message: String,
    pub last_activity: i64, 
    pub has_been_reminded: bool, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlacklistInfo {
    pub reason: String,
    pub by: u64, 
    pub date: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TicketStore {
    pub tickets: HashMap<u64, TicketInfo>,
    pub counts: HashMap<String, u32>, 
    #[serde(default)] 
    pub blacklist: HashMap<u64, BlacklistInfo>, 
}

impl TicketStore {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("tickets.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("tickets.json", content);
        }
    }
}
