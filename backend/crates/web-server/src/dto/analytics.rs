use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{Date, OffsetDateTime};
use validator::Validate;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsEventType {
    PageView,
    SessionStart,
    SessionEnd,
    FeatureClick,
    AgentQuestionSubmit,
    AgentAnswerView,
    ProcessPanelOpen,
    ReportView,
    ReportCopy,
    ReportDownload,
    FileUpload,
    LoginClick,
    LoginSuccess,
    RegisterSuccess,
    FrontendError,
    ApiError,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsSourceKind {
    Direct,
    Search,
    Referral,
    Social,
    Ad,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsDeviceType {
    Desktop,
    Mobile,
    Tablet,
    Bot,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsEventStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsGranularity {
    Day,
    Week,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsSortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalyticsAccountSortBy {
    LastSeenAt,
    Pv,
    Sessions,
    AgentQuestions,
    ActiveDays,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalyticsSchoolTeamSortBy {
    ActiveAccounts,
    Pv,
    Sessions,
    AgentQuestions,
    LastSeenAt,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TrackAnalyticsEventsRequest {
    #[validate(length(max = 20))]
    pub events: Vec<TrackAnalyticsEventRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TrackAnalyticsEventRequest {
    #[validate(length(min = 1, max = 128))]
    pub event_id: String,
    pub event_type: AnalyticsEventType,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    #[validate(length(max = 128))]
    pub anonymous_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub session_id: String,
    #[validate(length(max = 256))]
    pub route_path: Option<String>,
    #[validate(length(max = 256))]
    pub page_title: Option<String>,
    #[validate(length(max = 512))]
    pub referrer: Option<String>,
    pub source_kind: AnalyticsSourceKind,
    #[validate(length(max = 255))]
    pub source_host: Option<String>,
    #[validate(length(max = 128))]
    pub utm_source: Option<String>,
    #[validate(length(max = 128))]
    pub utm_medium: Option<String>,
    #[validate(length(max = 128))]
    pub utm_campaign: Option<String>,
    #[validate(length(max = 128))]
    pub utm_content: Option<String>,
    #[validate(length(max = 128))]
    pub utm_term: Option<String>,
    #[validate(length(max = 128))]
    pub feature_key: Option<String>,
    pub device_type: AnalyticsDeviceType,
    #[validate(length(max = 64))]
    pub browser_name: Option<String>,
    #[validate(length(max = 64))]
    pub os_name: Option<String>,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
    #[serde(default = "default_event_status")]
    pub status: AnalyticsEventStatus,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackAnalyticsEventsResponse {
    pub accepted: usize,
    pub duplicated: usize,
    pub rejected: usize,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsRangeQuery {
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub from: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub to: Option<OffsetDateTime>,
    pub granularity: Option<AnalyticsGranularity>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAnalyticsAccountsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(length(max = 128))]
    pub school_name: Option<String>,
    #[validate(length(max = 128))]
    pub team_name: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub from: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub to: Option<OffsetDateTime>,
    pub sort_by: Option<AnalyticsAccountSortBy>,
    pub sort_order: Option<AnalyticsSortOrder>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAnalyticsSchoolTeamsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[validate(length(max = 128))]
    pub school_name: Option<String>,
    #[validate(length(max = 128))]
    pub team_name: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub from: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub to: Option<OffsetDateTime>,
    pub sort_by: Option<AnalyticsSchoolTeamSortBy>,
    pub sort_order: Option<AnalyticsSortOrder>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsOverviewResponse {
    pub summary: AnalyticsSummary,
    pub daily_traffic: Vec<AnalyticsDailyTraffic>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub active_accounts: i64,
    pub agent_questions: i64,
    pub token_used: i64,
    pub avg_session_duration_sec: i64,
    pub bounce_rate: f64,
    pub frontend_errors: i64,
    pub api_errors: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDailyTraffic {
    pub date: Date,
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub active_accounts: i64,
    pub agent_questions: i64,
    pub token_used: i64,
    pub errors: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSourcesResponse {
    pub source_kinds: Vec<AnalyticsSourceMetric>,
    pub source_hosts: Vec<AnalyticsSourceMetric>,
    pub campaigns: Vec<AnalyticsSourceMetric>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSourceMetric {
    pub key: String,
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub agent_questions: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsPagesResponse {
    pub items: Vec<AnalyticsPageMetric>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsPageMetric {
    pub route_path: String,
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub avg_session_duration_sec: i64,
    pub entry_count: i64,
    pub exit_count: i64,
    pub errors: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsAccountRow {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: String,
    pub team_name: String,
    pub pv: i64,
    pub sessions: i64,
    pub active_days: i64,
    pub agent_questions: i64,
    pub token_used: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_seen_at: Option<OffsetDateTime>,
    pub top_pages: Vec<AnalyticsTopPage>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsTopPage {
    pub route_path: String,
    pub pv: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSchoolTeamRow {
    pub school_name: String,
    pub team_name: String,
    pub account_count: i64,
    pub active_accounts: i64,
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub agent_questions: i64,
    pub token_used: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_seen_at: Option<OffsetDateTime>,
}

fn default_event_status() -> AnalyticsEventStatus {
    AnalyticsEventStatus::Ok
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}
