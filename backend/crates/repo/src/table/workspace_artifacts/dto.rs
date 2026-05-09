use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceArtifactStatus {
    Active,
    Archived,
}

impl WorkspaceArtifactStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for WorkspaceArtifactStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace artifact status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for WorkspaceArtifactStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceArtifactValidationStatus {
    Unchecked,
    Passed,
    Failed,
}

impl WorkspaceArtifactValidationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for WorkspaceArtifactValidationStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "unchecked" => Ok(Self::Unchecked),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace artifact validation status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceArtifact {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Option<Uuid>,
    pub workspace_task_id: Option<Uuid>,
    pub run_task_id: Option<Uuid>,
    pub user_id: Uuid,
    pub artifact_type: String,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: String,
    pub storage_kind: String,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub source: String,
    pub status: WorkspaceArtifactStatus,
    pub validation_status: WorkspaceArtifactValidationStatus,
    pub validation_report: Value,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceArtifact {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Option<Uuid>,
    pub workspace_task_id: Option<Uuid>,
    pub run_task_id: Option<Uuid>,
    pub user_id: Uuid,
    pub artifact_type: String,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: String,
    pub storage_kind: String,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub source: String,
    pub status: WorkspaceArtifactStatus,
    pub validation_status: WorkspaceArtifactValidationStatus,
    pub validation_report: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceArtifact {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: String,
    pub storage_kind: String,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub status: WorkspaceArtifactStatus,
    pub validation_status: WorkspaceArtifactValidationStatus,
    pub validation_report: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceArtifactFilter {
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub workspace_task_id: Option<Uuid>,
    pub run_task_id: Option<Uuid>,
    pub user_id: Uuid,
    pub artifact_type: Option<String>,
    pub status: Option<WorkspaceArtifactStatus>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
