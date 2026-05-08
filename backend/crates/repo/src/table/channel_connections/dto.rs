use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelConnectionStatus {
    Configured,
    Disabled,
    Error,
}

impl ChannelConnectionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Configured => "configured",
            Self::Disabled => "disabled",
            Self::Error => "error",
        }
    }
}

impl TryFrom<&str> for ChannelConnectionStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "configured" => Ok(Self::Configured),
            "disabled" => Ok(Self::Disabled),
            "error" => Ok(Self::Error),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid channel connection status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub webhook_url: Option<String>,
    pub bot_token_enc: Option<String>,
    pub chat_id: Option<String>,
    pub secret_enc: Option<String>,
    pub extra_credentials: Value,
    pub enabled: bool,
    pub status: ChannelConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertChannelConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub webhook_url: Option<String>,
    pub bot_token_enc: Option<String>,
    pub chat_id: Option<String>,
    pub secret_enc: Option<String>,
    pub extra_credentials: Value,
    pub enabled: bool,
    pub status: ChannelConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChannelConnection {
    pub id: Uuid,
    pub channel: String,
    pub webhook_url: Option<String>,
    pub bot_token_enc: Option<String>,
    pub chat_id: Option<String>,
    pub secret_enc: Option<String>,
    pub extra_credentials: Value,
    pub enabled: bool,
    pub status: ChannelConnectionStatus,
    pub last_tested_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelConnectionFilter {
    pub user_id: Uuid,
    pub channel: Option<String>,
    pub enabled: Option<bool>,
}
