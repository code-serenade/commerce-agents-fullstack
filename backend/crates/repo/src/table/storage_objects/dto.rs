use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageObjectStatus {
    Active,
    Orphaned,
    Deleted,
}

impl StorageObjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Orphaned => "orphaned",
            Self::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObject {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub sha256: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub storage_provider: String,
    pub status: String,
    pub ref_count: i64,
    pub last_referenced_at: Option<OffsetDateTime>,
    pub orphaned_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStorageObject {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub sha256: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStorageObjectReferenceState {
    pub id: Uuid,
    pub status: StorageObjectStatus,
    pub ref_count: i64,
    pub last_referenced_at: Option<OffsetDateTime>,
    pub orphaned_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct StorageObjectCleanupFilter {
    pub limit: Option<u64>,
}
