use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Snoozed,
    Dismissed,
    Acted,
}

impl AlertStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Snoozed => "snoozed",
            Self::Dismissed => "dismissed",
            Self::Acted => "acted",
        }
    }
}

impl TryFrom<&str> for AlertStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "snoozed" => Ok(Self::Snoozed),
            "dismissed" => Ok(Self::Dismissed),
            "acted" => Ok(Self::Acted),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid alert status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub role: Option<String>,
    pub status: AlertStatus,
    pub dedup_key: Option<String>,
    pub snoozed_until: Option<OffsetDateTime>,
    pub payload: Value,
    pub acted_at: Option<OffsetDateTime>,
    pub dismissed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub role: Option<String>,
    pub status: AlertStatus,
    pub dedup_key: Option<String>,
    pub snoozed_until: Option<OffsetDateTime>,
    pub payload: Value,
    pub acted_at: Option<OffsetDateTime>,
    pub dismissed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlert {
    pub id: Uuid,
    pub status: AlertStatus,
    pub snoozed_until: Option<OffsetDateTime>,
    pub acted_at: Option<OffsetDateTime>,
    pub dismissed_at: Option<OffsetDateTime>,
    pub payload: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AlertFilter {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub alert_type: Option<String>,
    pub severity: Option<String>,
    pub status: Option<AlertStatus>,
    pub include_snoozed: bool,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
