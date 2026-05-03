use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub permission_code: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMenu {
    pub name: String,
    pub parent_id: Option<i64>,
    pub permission_code: Option<String>,
}
