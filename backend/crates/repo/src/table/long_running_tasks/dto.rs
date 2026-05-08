use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LongTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl LongTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for LongTaskStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid long task status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongRunningTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub task_type: String,
    pub status: LongTaskStatus,
    pub title: String,
    pub progress: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub input_params: Value,
    pub result: Value,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLongRunningTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub task_type: String,
    pub status: LongTaskStatus,
    pub title: String,
    pub progress: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub input_params: Value,
    pub result: Value,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLongRunningTask {
    pub id: Uuid,
    pub status: LongTaskStatus,
    pub title: String,
    pub progress: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub result: Value,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct LongRunningTaskFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub status: Option<LongTaskStatus>,
    pub task_type: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
