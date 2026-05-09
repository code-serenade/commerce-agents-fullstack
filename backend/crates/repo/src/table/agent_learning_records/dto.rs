use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentLearningRecordStatus {
    Active,
    Archived,
}

impl AgentLearningRecordStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for AgentLearningRecordStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid agent learning record status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLearningRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub quality_check_id: Option<Uuid>,
    pub role: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub source_action: String,
    pub feedback_rating: Option<i32>,
    pub feedback_comment: Option<String>,
    pub status: AgentLearningRecordStatus,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentLearningRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub quality_check_id: Option<Uuid>,
    pub role: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub source_action: String,
    pub feedback_rating: Option<i32>,
    pub feedback_comment: Option<String>,
    pub status: AgentLearningRecordStatus,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AgentLearningRecordFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub quality_check_id: Option<Uuid>,
    pub role: Option<String>,
    pub category: Option<String>,
    pub status: Option<AgentLearningRecordStatus>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
