use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRunTaskStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}

impl WorkspaceRunTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for WorkspaceRunTaskStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace run task status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRunTask {
    pub id: Uuid,
    pub run_id: Uuid,
    pub workspace_id: Uuid,
    pub workspace_task_id: Option<Uuid>,
    pub user_id: Uuid,
    pub seq: i32,
    pub owner_role: Option<String>,
    pub status: WorkspaceRunTaskStatus,
    pub attempts: i32,
    pub acceptance_score: f64,
    pub acceptance_passed: bool,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRunTask {
    pub id: Uuid,
    pub run_id: Uuid,
    pub workspace_id: Uuid,
    pub workspace_task_id: Option<Uuid>,
    pub user_id: Uuid,
    pub seq: i32,
    pub owner_role: Option<String>,
    pub status: WorkspaceRunTaskStatus,
    pub attempts: i32,
    pub acceptance_score: f64,
    pub acceptance_passed: bool,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceRunTask {
    pub id: Uuid,
    pub seq: i32,
    pub owner_role: Option<String>,
    pub status: WorkspaceRunTaskStatus,
    pub attempts: i32,
    pub acceptance_score: f64,
    pub acceptance_passed: bool,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceRunTaskFilter {
    pub run_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub user_id: Uuid,
    pub status: Option<WorkspaceRunTaskStatus>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
