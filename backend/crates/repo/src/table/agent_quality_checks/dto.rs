use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQualityCheck {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Uuid,
    pub artifact_id: Option<Uuid>,
    pub user_id: Uuid,
    pub role: String,
    pub score: f64,
    pub passed: bool,
    pub dimensions: Value,
    pub issues: Value,
    pub suggestions: Value,
    pub source: String,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentQualityCheck {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Uuid,
    pub artifact_id: Option<Uuid>,
    pub user_id: Uuid,
    pub role: String,
    pub score: f64,
    pub passed: bool,
    pub dimensions: Value,
    pub issues: Value,
    pub suggestions: Value,
    pub source: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AgentQualityCheckFilter {
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub artifact_id: Option<Uuid>,
    pub user_id: Uuid,
    pub role: Option<String>,
    pub passed: Option<bool>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
