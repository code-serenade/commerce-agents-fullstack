use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObject {
    pub sha256: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStorageObject {
    pub sha256: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub metadata: Value,
}
