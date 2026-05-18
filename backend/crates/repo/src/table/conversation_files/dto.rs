use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationFile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub message_id: Option<Uuid>,
    pub bucket: String,
    pub object_key: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub visibility: String,
    pub storage_provider: String,
    pub status: String,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversationFile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub message_id: Option<Uuid>,
    pub bucket: String,
    pub object_key: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConversationFileMetadata {
    pub id: Uuid,
    pub metadata: Value,
}
