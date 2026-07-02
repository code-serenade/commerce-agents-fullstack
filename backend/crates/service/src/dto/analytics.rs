use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsActorKind {
    Anonymous,
    AppUser,
}

impl AnalyticsActorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::AppUser => "app_user",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
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

impl AnalyticsEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PageView => "page_view",
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::FeatureClick => "feature_click",
            Self::AgentQuestionSubmit => "agent_question_submit",
            Self::AgentAnswerView => "agent_answer_view",
            Self::ProcessPanelOpen => "process_panel_open",
            Self::ReportView => "report_view",
            Self::ReportCopy => "report_copy",
            Self::ReportDownload => "report_download",
            Self::FileUpload => "file_upload",
            Self::LoginClick => "login_click",
            Self::LoginSuccess => "login_success",
            Self::RegisterSuccess => "register_success",
            Self::FrontendError => "frontend_error",
            Self::ApiError => "api_error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
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

impl AnalyticsSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Search => "search",
            Self::Referral => "referral",
            Self::Social => "social",
            Self::Ad => "ad",
            Self::Internal => "internal",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsDeviceType {
    Desktop,
    Mobile,
    Tablet,
    Bot,
    Unknown,
}

impl AnalyticsDeviceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Tablet => "tablet",
            Self::Bot => "bot",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsEventStatus {
    Ok,
    Failed,
}

impl AnalyticsEventStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Clone)]
pub struct TrackAnalyticsEventsRequest {
    pub actor_kind: AnalyticsActorKind,
    pub user_id: Option<uuid::Uuid>,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub events: Vec<TrackAnalyticsEventInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackAnalyticsEventInput {
    pub event_id: String,
    pub event_type: AnalyticsEventType,
    pub occurred_at: OffsetDateTime,
    pub anonymous_id: Option<String>,
    pub session_id: String,
    pub route_path: Option<String>,
    pub page_title: Option<String>,
    pub referrer: Option<String>,
    pub source_kind: AnalyticsSourceKind,
    pub source_host: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub feature_key: Option<String>,
    pub device_type: AnalyticsDeviceType,
    pub browser_name: Option<String>,
    pub os_name: Option<String>,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
    pub status: AnalyticsEventStatus,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackAnalyticsEventsResponse {
    pub accepted: usize,
    pub duplicated: usize,
    pub rejected: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsRangeRequest {
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub granularity: Option<AnalyticsGranularity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsOverviewResponse {
    pub summary: AnalyticsSummary,
    pub daily_traffic: Vec<AnalyticsDailyTraffic>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSourcesResponse {
    pub source_kinds: Vec<AnalyticsSourceMetric>,
    pub source_hosts: Vec<AnalyticsSourceMetric>,
    pub campaigns: Vec<AnalyticsSourceMetric>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSourceMetric {
    pub key: String,
    pub pv: i64,
    pub uv: i64,
    pub sessions: i64,
    pub agent_questions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsPagesResponse {
    pub items: Vec<AnalyticsPageMetric>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ListAnalyticsAccountsRequest {
    pub page: u64,
    pub page_size: u64,
    pub keyword: Option<String>,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub sort_by: Option<AnalyticsAccountSortBy>,
    pub sort_order: Option<AnalyticsSortOrder>,
}

#[derive(Debug, Clone, Serialize)]
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
    pub last_seen_at: Option<OffsetDateTime>,
    pub top_pages: Vec<AnalyticsTopPage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsTopPage {
    pub route_path: String,
    pub pv: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAnalyticsSchoolTeamsRequest {
    pub page: u64,
    pub page_size: u64,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
    pub sort_by: Option<AnalyticsSchoolTeamSortBy>,
    pub sort_order: Option<AnalyticsSortOrder>,
}

#[derive(Debug, Clone, Serialize)]
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
    pub last_seen_at: Option<OffsetDateTime>,
}
