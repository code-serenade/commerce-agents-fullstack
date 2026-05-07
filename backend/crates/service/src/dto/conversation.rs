use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversationRequest {
    pub user_id: String,
    pub title: Option<String>,
    pub first_message: Option<String>,
    pub agent_id: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddConversationMessageRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub role: String,
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConversationRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteConversationRequest {
    pub user_id: String,
    pub conversation_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterConversationFileRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub message_id: Option<String>,
    pub object_key: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub status: String,
    pub source: String,
    pub agent_id: Option<String>,
    pub model_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationDetailResponse {
    pub conversation: ConversationResponse,
    pub messages: Vec<ConversationMessageResponse>,
    pub files: Vec<ConversationFileResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationMessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub user_id: String,
    pub role: String,
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
    pub status: String,
    pub sequence: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationFileResponse {
    pub id: String,
    pub conversation_id: String,
    pub message_id: Option<String>,
    pub bucket: String,
    pub object_key: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationFileAccessResponse {
    pub file: ConversationFileResponse,
    pub method: String,
    pub download_url: String,
    pub expires_secs: u64,
}

pub(crate) fn uuid_to_string(value: Uuid) -> String {
    value.to_string()
}
