use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformConnectionStatus {
    Configured,
    Disabled,
    Error,
}

impl PlatformConnectionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Configured => "configured",
            Self::Disabled => "disabled",
            Self::Error => "error",
        }
    }
}

impl TryFrom<&str> for PlatformConnectionStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "configured" => Ok(Self::Configured),
            "disabled" => Ok(Self::Disabled),
            "error" => Ok(Self::Error),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid platform connection status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformSyncStatus {
    Idle,
    Syncing,
    Success,
    Failed,
}

impl PlatformSyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Syncing => "syncing",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for PlatformSyncStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "idle" => Ok(Self::Idle),
            "syncing" => Ok(Self::Syncing),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid platform sync status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub app_key: Option<String>,
    pub app_secret_enc: Option<String>,
    pub access_token_enc: Option<String>,
    pub refresh_token_enc: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: bool,
    pub status: PlatformConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub test_result: Value,
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: PlatformSyncStatus,
    pub sync_message: Option<String>,
    pub extra: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPlatformConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub app_key: Option<String>,
    pub app_secret_enc: Option<String>,
    pub access_token_enc: Option<String>,
    pub refresh_token_enc: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: bool,
    pub status: PlatformConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub test_result: Value,
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: PlatformSyncStatus,
    pub sync_message: Option<String>,
    pub extra: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePlatformConnection {
    pub id: Uuid,
    pub platform: String,
    pub app_key: Option<String>,
    pub app_secret_enc: Option<String>,
    pub access_token_enc: Option<String>,
    pub refresh_token_enc: Option<String>,
    pub token_expires_at: Option<OffsetDateTime>,
    pub shop_id: Option<String>,
    pub shop_name: Option<String>,
    pub enabled: bool,
    pub status: PlatformConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub test_result: Value,
    pub last_synced_at: Option<OffsetDateTime>,
    pub sync_status: PlatformSyncStatus,
    pub sync_message: Option<String>,
    pub extra: Value,
}

#[derive(Debug, Clone, Default)]
pub struct PlatformConnectionFilter {
    pub user_id: Uuid,
    pub platform: Option<String>,
    pub enabled: Option<bool>,
}
