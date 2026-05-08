use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Cancelled,
    Archived,
}

impl WorkspaceTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Cancelled => "cancelled",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for WorkspaceTaskStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "blocked" => Ok(Self::Blocked),
            "cancelled" => Ok(Self::Cancelled),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace task status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTask {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: WorkspaceTaskStatus,
    pub priority: i32,
    pub depends_on: Value,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceTask {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: WorkspaceTaskStatus,
    pub priority: i32,
    pub depends_on: Value,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceTask {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: WorkspaceTaskStatus,
    pub priority: i32,
    pub depends_on: Value,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceTaskFilter {
    pub workspace_id: Option<Uuid>,
    pub user_id: Uuid,
    pub status: Option<WorkspaceTaskStatus>,
    pub owner_role: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
