use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Group,
    Action,
}

impl PermissionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Action => "action",
        }
    }
}

impl TryFrom<&str> for PermissionKind {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "group" => Ok(Self::Group),
            "action" => Ok(Self::Action),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid permission kind: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for PermissionKind {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub sort: i32,
    pub kind: PermissionKind,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePermission {
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub sort: i32,
    pub kind: PermissionKind,
}
