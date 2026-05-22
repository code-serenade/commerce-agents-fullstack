use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageObjectRefStatus {
    Active,
    Released,
}

impl StorageObjectRefStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Released => "released",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObjectRef {
    pub id: Uuid,
    pub storage_object_id: Uuid,
    pub user_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub status: String,
    pub released_at: Option<OffsetDateTime>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStorageObjectRef {
    pub id: Uuid,
    pub storage_object_id: Uuid,
    pub user_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStorageObjectRefStatus {
    pub id: Uuid,
    pub status: StorageObjectRefStatus,
    pub released_at: Option<OffsetDateTime>,
}
