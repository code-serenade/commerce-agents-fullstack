use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamUsageQuotaStatus {
    Enabled,
    Disabled,
}

impl TeamUsageQuotaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for TeamUsageQuotaStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid team usage quota status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for TeamUsageQuotaStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUsageQuota {
    pub id: Uuid,
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub token_limit: Option<i32>,
    pub status: TeamUsageQuotaStatus,
    pub remark: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamUsageQuota {
    pub id: Uuid,
    pub school_name: String,
    pub team_name: String,
    pub question_limit: i32,
    pub token_limit: Option<i32>,
    pub status: TeamUsageQuotaStatus,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamUsageQuota {
    pub id: Uuid,
    pub question_limit: i32,
    pub token_limit: Option<i32>,
    pub status: TeamUsageQuotaStatus,
    pub remark: Option<String>,
}
