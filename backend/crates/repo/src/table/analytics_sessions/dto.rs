use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSession {
    pub id: Uuid,
    pub session_id: String,
    pub actor_kind: String,
    pub user_id: Option<Uuid>,
    pub anonymous_id: Option<String>,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub first_seen_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
    pub entry_path: Option<String>,
    pub exit_path: Option<String>,
    pub source_kind: String,
    pub source_host: Option<String>,
    pub page_view_count: i32,
    pub event_count: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnalyticsSession {
    pub id: Uuid,
    pub session_id: String,
    pub actor_kind: String,
    pub user_id: Option<Uuid>,
    pub anonymous_id: Option<String>,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub first_seen_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
    pub entry_path: Option<String>,
    pub exit_path: Option<String>,
    pub source_kind: String,
    pub source_host: Option<String>,
    pub page_view_count: i32,
    pub event_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAnalyticsSession {
    pub id: Uuid,
    pub actor_kind: String,
    pub user_id: Option<Uuid>,
    pub anonymous_id: Option<String>,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub last_seen_at: OffsetDateTime,
    pub exit_path: Option<String>,
    pub source_kind: String,
    pub source_host: Option<String>,
    pub page_view_count: i32,
    pub event_count: i32,
}
