use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageObjectAnalysisStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
}

impl StorageObjectAnalysisStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObjectAnalysis {
    pub id: Uuid,
    pub storage_object_sha256: String,
    pub analysis_kind: String,
    pub status: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
    pub summary: Option<String>,
    pub result_json: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStorageObjectAnalysis {
    pub id: Uuid,
    pub storage_object_sha256: String,
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageObjectAnalysisIdentity {
    pub storage_object_sha256: String,
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStorageObjectAnalysisProcessing {
    pub id: Uuid,
    pub llm_model: Option<String>,
    pub started_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStorageObjectAnalysisSucceeded {
    pub id: Uuid,
    pub llm_model: Option<String>,
    pub summary: String,
    pub result_json: Value,
    pub finished_at: OffsetDateTime,
}
