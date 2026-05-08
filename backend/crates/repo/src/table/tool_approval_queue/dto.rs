use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolApprovalStatus {
    Pending,
    Approved,
    Executed,
    Failed,
    Rejected,
    Cancelled,
}

impl ToolApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Executed => "executed",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for ToolApprovalStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "executed" => Ok(Self::Executed),
            "failed" => Ok(Self::Failed),
            "rejected" => Ok(Self::Rejected),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid tool approval status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolApproval {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub request_role: Option<String>,
    pub skill_name: String,
    pub tool_call_id: Option<String>,
    pub risk_level: String,
    pub args: Value,
    pub status: ToolApprovalStatus,
    pub reason: Option<String>,
    pub action_key: Option<String>,
    pub result: Value,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<OffsetDateTime>,
    pub executed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateToolApproval {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub request_role: Option<String>,
    pub skill_name: String,
    pub tool_call_id: Option<String>,
    pub risk_level: String,
    pub args: Value,
    pub status: ToolApprovalStatus,
    pub reason: Option<String>,
    pub action_key: Option<String>,
    pub result: Value,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<OffsetDateTime>,
    pub executed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateToolApprovalDecision {
    pub id: Uuid,
    pub status: ToolApprovalStatus,
    pub reason: Option<String>,
    pub result: Value,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<OffsetDateTime>,
    pub executed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct ToolApprovalFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub status: Option<ToolApprovalStatus>,
    pub skill_name: Option<String>,
    pub risk_level: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
