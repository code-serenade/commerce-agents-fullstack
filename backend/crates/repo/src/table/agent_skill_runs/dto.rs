use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSkillRunStatus {
    Requested,
    Success,
    Deferred,
    PendingApproval,
    NotFound,
    ValidationError,
    Failed,
}

impl AgentSkillRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Success => "success",
            Self::Deferred => "deferred",
            Self::PendingApproval => "pending_approval",
            Self::NotFound => "not_found",
            Self::ValidationError => "validation_error",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for AgentSkillRunStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "requested" => Ok(Self::Requested),
            "success" => Ok(Self::Success),
            "deferred" => Ok(Self::Deferred),
            "pending_approval" => Ok(Self::PendingApproval),
            "not_found" => Ok(Self::NotFound),
            "validation_error" => Ok(Self::ValidationError),
            "failed" => Ok(Self::Failed),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid agent skill run status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillRun {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Uuid,
    pub session_id: Option<String>,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub status: AgentSkillRunStatus,
    pub reason: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub input: Value,
    pub output: Value,
    pub execution_mode: Option<String>,
    pub risk_level: Option<String>,
    pub approval_required: bool,
    pub metadata: Value,
    pub requested_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentSkillRun {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Uuid,
    pub session_id: Option<String>,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub status: AgentSkillRunStatus,
    pub reason: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub input: Value,
    pub output: Value,
    pub execution_mode: Option<String>,
    pub risk_level: Option<String>,
    pub approval_required: bool,
    pub metadata: Value,
    pub requested_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAgentSkillRun {
    pub id: Uuid,
    pub status: AgentSkillRunStatus,
    pub reason: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub input: Value,
    pub output: Value,
    pub execution_mode: Option<String>,
    pub risk_level: Option<String>,
    pub approval_required: bool,
    pub metadata: Value,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentSkillRunFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub status: Option<AgentSkillRunStatus>,
    pub skill_id: Option<String>,
    pub role: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
