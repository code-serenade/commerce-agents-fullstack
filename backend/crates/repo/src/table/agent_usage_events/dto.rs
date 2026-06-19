use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_name: String,
    pub team_name: String,
    pub conversation_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub question_count: i32,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    pub status: String,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentUsageEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_name: String,
    pub team_name: String,
    pub conversation_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub question_count: i32,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
    pub status: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUsageEventStats {
    pub school_name: String,
    pub team_name: String,
    pub question_used: i64,
    pub token_used: i64,
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsageEventStats {
    pub user_id: Uuid,
    pub question_used: i64,
    pub token_used: i64,
    pub last_used_at: Option<OffsetDateTime>,
}
