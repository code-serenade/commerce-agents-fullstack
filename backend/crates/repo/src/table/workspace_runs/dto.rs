use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRunStatus {
    Planned,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Archived,
}

impl WorkspaceRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for WorkspaceRunStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "planned" => Ok(Self::Planned),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace run status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRun {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub goal: String,
    pub requirements: Value,
    pub status: WorkspaceRunStatus,
    pub source: String,
    pub summary: Option<String>,
    pub report: Value,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRun {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub goal: String,
    pub requirements: Value,
    pub status: WorkspaceRunStatus,
    pub source: String,
    pub summary: Option<String>,
    pub report: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceRun {
    pub id: Uuid,
    pub goal: String,
    pub requirements: Value,
    pub status: WorkspaceRunStatus,
    pub source: String,
    pub summary: Option<String>,
    pub report: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceRunFilter {
    pub workspace_id: Option<Uuid>,
    pub user_id: Uuid,
    pub status: Option<WorkspaceRunStatus>,
    pub source: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
