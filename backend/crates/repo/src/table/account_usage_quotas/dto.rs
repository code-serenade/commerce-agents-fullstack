use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountUsageQuotaStatus {
    Enabled,
    Disabled,
}

impl AccountUsageQuotaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for AccountUsageQuotaStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid account usage quota status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for AccountUsageQuotaStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUsageQuota {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question_limit: i32,
    pub status: AccountUsageQuotaStatus,
    pub remark: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountUsageQuota {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question_limit: i32,
    pub status: AccountUsageQuotaStatus,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountUsageQuota {
    pub id: Uuid,
    pub question_limit: i32,
    pub status: AccountUsageQuotaStatus,
    pub remark: Option<String>,
}
