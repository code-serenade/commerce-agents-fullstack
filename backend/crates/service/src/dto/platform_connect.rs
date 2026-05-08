use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFieldSpec {
    pub key: String,
    pub label: String,
    pub secret: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSpec {
    pub platform: String,
    pub display: String,
    pub required_fields: Vec<PlatformFieldSpec>,
    pub doc_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCatalogResponse {
    pub platforms: Vec<PlatformSpec>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSpec {
    pub channel: String,
    pub display: String,
    pub required_fields: Vec<PlatformFieldSpec>,
    pub doc_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCatalogResponse {
    pub channels: Vec<ChannelSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPlatformConnectionsRequest {
    pub user_id: String,
    pub platform: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavePlatformConnectionRequest {
    pub user_id: String,
    pub platform: String,
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: Option<bool>,
    pub test_result: Option<Value>,
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: Option<String>,
    pub sync_message: Option<String>,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPlatformConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePlatformConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
    pub platform: Option<String>,
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: Option<bool>,
    pub test_result: Option<Value>,
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: Option<String>,
    pub sync_message: Option<String>,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisablePlatformConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformConnectionResponse {
    pub id: String,
    pub user_id: String,
    pub platform: String,
    pub display: String,
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: bool,
    pub status: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_tested_at: Option<OffsetDateTime>,
    pub test_result: Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: String,
    pub sync_message: Option<String>,
    pub extra: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformConnectionListResponse {
    pub connections: Vec<PlatformConnectionResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListChannelConnectionsRequest {
    pub user_id: String,
    pub channel: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveChannelConnectionRequest {
    pub user_id: String,
    pub channel: String,
    pub webhook_url: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub secret: Option<String>,
    pub extra_credentials: Option<Value>,
    pub enabled: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetChannelConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChannelConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
    pub channel: Option<String>,
    pub webhook_url: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub secret: Option<String>,
    pub extra_credentials: Option<Value>,
    pub enabled: Option<bool>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisableChannelConnectionRequest {
    pub user_id: String,
    pub connection_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelConnectionResponse {
    pub id: String,
    pub user_id: String,
    pub channel: String,
    pub display: String,
    pub webhook_url: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub secret: Option<String>,
    pub enabled: bool,
    pub status: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_tested_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub extra_credentials: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelConnectionListResponse {
    pub connections: Vec<ChannelConnectionResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListStoreMetricsRequest {
    pub user_id: String,
    pub platform: Option<String>,
    pub metric_key: Option<String>,
    pub metric_date_from: Option<String>,
    pub metric_date_to: Option<String>,
    pub source: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveStoreMetricRequest {
    pub user_id: String,
    pub platform: String,
    pub metric_date: String,
    pub metric_key: String,
    pub metric_value: f64,
    pub dimension_key: Option<String>,
    pub dimension_value: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMetricResponse {
    pub id: String,
    pub user_id: String,
    pub platform: String,
    pub metric_date: String,
    pub metric_key: String,
    pub metric_value: f64,
    pub dimension_key: String,
    pub dimension_value: String,
    pub source: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMetricListResponse {
    pub metrics: Vec<StoreMetricResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSyncStatusResponse {
    pub platform: String,
    pub display: String,
    pub connected: bool,
    pub enabled: bool,
    pub sync_status: String,
    pub sync_message: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_tested_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_synced_at: Option<OffsetDateTime>,
    pub latest_metric_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSyncStatusListResponse {
    pub platforms: Vec<PlatformSyncStatusResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSummaryItem {
    pub platform: String,
    pub display: String,
    pub connected: bool,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_tested_at: Option<OffsetDateTime>,
    pub gmv_7d: f64,
    pub orders_7d: f64,
    pub gmv_30d: f64,
    pub orders_30d: f64,
    pub uv_30d: f64,
    pub ad_roi_30d: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSummaryResponse {
    pub platforms: Vec<PlatformSummaryItem>,
}
