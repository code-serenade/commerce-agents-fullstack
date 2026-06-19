use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeamUsageStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TeamUsageSortBy {
    UpdatedAt,
    QuestionUsed,
    QuestionRemaining,
    MemberCount,
    LastUsedAt,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListTeamUsagesQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    pub status: Option<TeamUsageStatus>,
    pub sort_by: Option<TeamUsageSortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeamUsageQuotaRequest {
    #[validate(range(min = 0, max = 1000000))]
    pub question_limit: i32,
    pub status: TeamUsageStatus,
    #[validate(length(max = 255))]
    pub remark: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamUsageResponse {
    pub id: String,
    pub school_name: String,
    pub team_name: String,
    pub member_count: u64,
    pub question_limit: i32,
    pub question_used: i64,
    pub question_remaining: i64,
    pub token_used: i64,
    pub status: TeamUsageStatus,
    pub remark: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamUsageMemberResponse {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub status: AppUserStatus,
    pub question_used: i64,
    pub token_used: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentTeamUsageQuotaResponse {
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub question_used: i64,
    pub question_remaining: i64,
    pub status: TeamUsageStatus,
    pub policy_text: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppUserStatus {
    Enabled,
    Disabled,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}
