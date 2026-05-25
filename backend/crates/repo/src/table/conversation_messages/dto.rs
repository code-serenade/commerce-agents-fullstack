use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
    pub status: String,
    pub sequence: i64,
    pub token_count: Option<i32>,
    pub metadata: Value,
    pub strategy_id: Option<String>,
    pub response_mode: Option<String>,
    pub routing_mode: Option<String>,
    pub collaboration_mode: Option<String>,
    pub learning_level: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversationMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
    pub sequence: i64,
    pub token_count: Option<i32>,
    pub metadata: Value,
    pub strategy_id: Option<String>,
    pub response_mode: Option<String>,
    pub routing_mode: Option<String>,
    pub collaboration_mode: Option<String>,
    pub learning_level: Option<String>,
}
