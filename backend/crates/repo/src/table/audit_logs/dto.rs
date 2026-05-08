use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub actor_user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub execution_queue_id: Option<Uuid>,
    pub tool_approval_id: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub actor_user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub execution_queue_id: Option<Uuid>,
    pub tool_approval_id: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub user_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub execution_queue_id: Option<Uuid>,
    pub tool_approval_id: Option<Uuid>,
    pub action: Option<String>,
    pub target_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
