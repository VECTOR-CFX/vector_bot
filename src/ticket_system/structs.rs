use serde::{Deserialize, Serialize};
use poise::serenity_prelude as serenity;
use sqlx::FromRow;

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

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct TicketInfo {
    pub user_id: i64,
    pub channel_id: i64,
    pub category: String,
    pub created_at: i64,
    pub initial_message: String,
    pub last_activity: i64,
    pub has_been_reminded: bool,
}
