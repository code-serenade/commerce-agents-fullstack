use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminUserStatus {
    Enabled,
    Disabled,
}

impl AdminUserStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for AdminUserStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid admin user status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for AdminUserStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub user_id: Uuid,
    pub display_id: String,
    pub display_name: String,
    pub remark: Option<String>,
    pub status: AdminUserStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAdminUser {
    pub user_id: Uuid,
    pub display_id: String,
    pub display_name: String,
    pub remark: Option<String>,
    pub status: AdminUserStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAdminUser {
    pub user_id: Uuid,
    pub remark: Option<String>,
    pub status: AdminUserStatus,
}
