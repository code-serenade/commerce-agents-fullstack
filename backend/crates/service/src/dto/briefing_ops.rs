use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListLongTasksRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub status: Option<String>,
    pub task_type: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLongTaskRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub task_type: String,
    pub title: String,
    pub status: Option<String>,
    pub progress: Option<i32>,
    pub total_steps: Option<i32>,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub input_params: Option<Value>,
    pub result: Option<Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetLongTaskRequest {
    pub user_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLongTaskRequest {
    pub user_id: String,
    pub task_id: String,
    pub status: Option<String>,
    pub title: Option<String>,
    pub progress: Option<i32>,
    pub total_steps: Option<i32>,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelLongTaskRequest {
    pub user_id: String,
    pub task_id: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDailyBriefingRequest {
    pub user_id: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDailyBriefingRecordsRequest {
    pub user_id: String,
    pub archived: Option<bool>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertDailyBriefingRecordRequest {
    pub user_id: String,
    pub briefing_date: String,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Option<Value>,
    pub product_summary: Option<Value>,
    pub alert_summary: Option<Value>,
    pub task_summary: Option<Value>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDailyBriefingRecordRequest {
    pub user_id: String,
    pub briefing_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveDailyBriefingRecordRequest {
    pub user_id: String,
    pub briefing_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListBackgroundEventsRequest {
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub event_type: Option<String>,
    pub consumed: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBackgroundEventRequest {
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub event_type: String,
    pub title: Option<String>,
    pub data: Option<Value>,
    pub consumed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumeBackgroundEventRequest {
    pub user_id: String,
    pub event_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumeBackgroundEventsRequest {
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAlertsRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub alert_type: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub include_snoozed: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlertRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub alert_type: String,
    pub severity: Option<String>,
    pub message: String,
    pub role: Option<String>,
    pub dedup_key: Option<String>,
    pub snoozed_until: Option<OffsetDateTime>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetAlertRequest {
    pub user_id: String,
    pub alert_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnoozeAlertRequest {
    pub user_id: String,
    pub alert_id: String,
    pub snoozed_until: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DismissAlertRequest {
    pub user_id: String,
    pub alert_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActAlertRequest {
    pub user_id: String,
    pub alert_id: String,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DismissAllAlertsRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub alert_type: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LongTaskListResponse {
    pub items: Vec<LongTaskResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LongTaskResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub task_type: String,
    pub status: String,
    pub title: String,
    pub progress: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub role: Option<String>,
    pub input_params: Value,
    pub result: Value,
    pub error_message: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBriefingSummaryResponse {
    pub date: String,
    pub generated: bool,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Value,
    pub product_summary: Value,
    pub alert_summary: Value,
    pub task_summary: Value,
    pub source: String,
    pub briefing: Option<DailyBriefingResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBriefingListResponse {
    pub records: Vec<DailyBriefingResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBriefingResponse {
    pub id: String,
    pub user_id: String,
    pub briefing_date: String,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Value,
    pub product_summary: Value,
    pub alert_summary: Value,
    pub task_summary: Value,
    pub source: String,
    pub status: String,
    pub archived: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundEventListResponse {
    pub events: Vec<BackgroundEventResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundEventResponse {
    pub id: String,
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub event_type: String,
    pub title: Option<String>,
    pub data: Value,
    pub consumed: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub consumed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub updated: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertListResponse {
    pub alerts: Vec<AlertResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertSummaryResponse {
    pub active: u64,
    pub by_severity: Value,
    pub by_type: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub role: Option<String>,
    pub status: String,
    pub dedup_key: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub snoozed_until: Option<OffsetDateTime>,
    pub payload: Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub acted_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub dismissed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
