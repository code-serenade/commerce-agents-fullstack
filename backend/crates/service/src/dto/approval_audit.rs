use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListToolApprovalsRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub status: Option<String>,
    pub skill_name: Option<String>,
    pub risk_level: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateToolApprovalRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub conversation_id: Option<String>,
    pub request_role: Option<String>,
    pub skill_name: String,
    pub tool_call_id: Option<String>,
    pub risk_level: Option<String>,
    pub args: Option<Value>,
    pub action_key: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetToolApprovalRequest {
    pub user_id: String,
    pub approval_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecideToolApprovalRequest {
    pub user_id: String,
    pub approval_id: String,
    pub reason: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAuditLogsRequest {
    pub user_id: String,
    pub actor_user_id: Option<String>,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub execution_queue_id: Option<String>,
    pub tool_approval_id: Option<String>,
    pub action: Option<String>,
    pub target_type: Option<String>,
    pub status: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditLogRequest {
    pub user_id: String,
    pub actor_user_id: Option<String>,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub execution_queue_id: Option<String>,
    pub tool_approval_id: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub status: Option<String>,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApprovalListResponse {
    pub items: Vec<ToolApprovalResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApprovalResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub conversation_id: Option<String>,
    pub request_role: Option<String>,
    pub skill_name: String,
    pub tool_call_id: Option<String>,
    pub risk_level: String,
    pub args: Value,
    pub status: String,
    pub reason: Option<String>,
    pub action_key: Option<String>,
    pub result: Value,
    pub approved_by: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub approved_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub executed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogResponse {
    pub id: String,
    pub user_id: String,
    pub actor_user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub execution_queue_id: Option<String>,
    pub tool_approval_id: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
