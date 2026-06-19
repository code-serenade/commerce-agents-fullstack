use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub use repo::table::account_usage_quotas::AccountUsageQuotaStatus as TeamUsageQuotaStatus;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TeamUsageSortBy {
    UpdatedAt,
    DisplayId,
    QuestionUsed,
    QuestionRemaining,
    LastUsedAt,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListTeamUsagesRequest {
    pub page: u64,
    pub page_size: u64,
    pub keyword: Option<String>,
    pub status: Option<TeamUsageQuotaStatus>,
    pub sort_by: Option<TeamUsageSortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamUsageQuotaRequest {
    pub usage_id: String,
    pub question_limit: i32,
    pub status: TeamUsageQuotaStatus,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamUsageResponse {
    pub id: String,
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub question_used: i64,
    pub question_remaining: i64,
    pub token_used: i64,
    pub status: TeamUsageQuotaStatus,
    pub remark: Option<String>,
    pub last_used_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct AgentQuestionPreflightRequest {
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub session_id: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentQuestionPreflight {
    pub user_id: uuid::Uuid,
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub question_used: i64,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub conversation_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone)]
pub struct RecordAgentQuestionStartedRequest {
    pub preflight: AgentQuestionPreflight,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub input: String,
}

#[derive(Debug, Clone)]
pub struct RecordAgentQuestionOutputRequest {
    pub preflight: AgentQuestionPreflight,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub output: Option<String>,
    pub status: AgentQuestionOutputStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum AgentQuestionOutputStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentTeamUsageQuotaResponse {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub question_used: i64,
    pub question_remaining: i64,
    pub status: TeamUsageQuotaStatus,
    pub policy_text: String,
}
