use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub status: String,
    pub source: String,
    pub agent_id: Option<String>,
    pub model_name: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub source: String,
    pub agent_id: Option<String>,
    pub model_name: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConversation {
    pub id: Uuid,
    pub title: Option<String>,
    pub status: String,
    pub archived_at: Option<OffsetDateTime>,
}
