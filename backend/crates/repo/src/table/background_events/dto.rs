use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub event_type: String,
    pub title: Option<String>,
    pub data: Value,
    pub consumed: bool,
    pub consumed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBackgroundEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub event_type: String,
    pub title: Option<String>,
    pub data: Value,
    pub consumed: bool,
    pub consumed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBackgroundEventConsumed {
    pub id: Uuid,
    pub consumed: bool,
    pub consumed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct BackgroundEventFilter {
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub consumed: Option<bool>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
