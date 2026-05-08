use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionQueueStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Failed,
    Cancelled,
}

impl ExecutionQueueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Executed => "executed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for ExecutionQueueStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "executed" => Ok(Self::Executed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid execution queue status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub workspace_task_id: Option<Uuid>,
    pub action_type: String,
    pub description: String,
    pub platform: Option<String>,
    pub payload: Value,
    pub expected_impact: Option<String>,
    pub status: ExecutionQueueStatus,
    pub decision_reason: Option<String>,
    pub result: Value,
    pub created_at: OffsetDateTime,
    pub decided_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExecutionQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub workspace_task_id: Option<Uuid>,
    pub action_type: String,
    pub description: String,
    pub platform: Option<String>,
    pub payload: Value,
    pub expected_impact: Option<String>,
    pub status: ExecutionQueueStatus,
    pub decision_reason: Option<String>,
    pub result: Value,
    pub decided_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateExecutionQueueItem {
    pub id: Uuid,
    pub status: ExecutionQueueStatus,
    pub decision_reason: Option<String>,
    pub result: Value,
    pub decided_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionQueueFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub status: Option<ExecutionQueueStatus>,
    pub action_type: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
