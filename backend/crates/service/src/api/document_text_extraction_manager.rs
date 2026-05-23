use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use repo::table::{
    conversation_files::{ConversationFile, ConversationFileService},
    storage_object_analyses::StorageObjectAnalysis,
    storage_object_refs::StorageObjectRefService,
    storage_objects::{StorageObject, StorageObjectService},
};
use uuid::Uuid;

use crate::api::storage_object_analysis::{
    EnsureStorageObjectAnalysisRequest, StorageObjectAnalysisApi,
};

const DOCUMENT_TEXT_EXTRACTION_MANAGER_ERROR: i16 = -1;
pub const DOCUMENT_TEXT_EXTRACTION_ANALYSIS_KIND: &str = "document_text_extraction";
pub const DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY: &str = "file-ingest";
pub const DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION: &str = "0.1.1";
pub const DOCUMENT_TEXT_EXTRACTION_LLM_NAME: &str = "none";

#[derive(Debug, Clone)]
pub struct DocumentTextExtractionTask {
    pub file: ConversationFile,
    pub object: StorageObject,
    pub analysis: StorageObjectAnalysis,
}

pub struct DocumentTextExtractionManager {
    file_svc: ConversationFileService,
    ref_svc: StorageObjectRefService,
    object_svc: StorageObjectService,
    analysis_api: StorageObjectAnalysisApi,
}

impl DocumentTextExtractionManager {
    pub fn new(db: DbContext) -> Self {
        Self {
            file_svc: ConversationFileService::new(db.clone()),
            ref_svc: StorageObjectRefService::new(db.clone()),
            object_svc: StorageObjectService::new(db.clone()),
            analysis_api: StorageObjectAnalysisApi::new(db),
        }
    }

    pub async fn ensure_task_for_conversation_file(
        &self,
        conversation_file_id: Uuid,
    ) -> BizResult<Option<DocumentTextExtractionTask>> {
        let Some(file) = self.file_svc.get_by_id(conversation_file_id).await? else {
            return Err(BizError::new(
                DOCUMENT_TEXT_EXTRACTION_MANAGER_ERROR,
                "conversation file not found".to_string(),
            ));
        };
        if file.status != "active" || !is_supported_document_text_file(&file) {
            return Ok(None);
        }

        let (object, analysis) = self.ensure_analysis_for_file(file.id).await?;

        Ok(Some(DocumentTextExtractionTask {
            file,
            object,
            analysis,
        }))
    }

    pub async fn latest_succeeded_for_file(
        &self,
        conversation_file_id: Uuid,
    ) -> BizResult<Option<StorageObjectAnalysis>> {
        let Some(storage_ref) = self
            .ref_svc
            .get_active_by_source("conversation_file", conversation_file_id)
            .await?
        else {
            return Ok(None);
        };

        self.analysis_api
            .latest_succeeded_for_object(
                &storage_ref.storage_object_sha256,
                DOCUMENT_TEXT_EXTRACTION_ANALYSIS_KIND,
            )
            .await
    }

    pub async fn ensure_analysis_for_file(
        &self,
        conversation_file_id: Uuid,
    ) -> BizResult<(StorageObject, StorageObjectAnalysis)> {
        let Some(storage_ref) = self
            .ref_svc
            .get_active_by_source("conversation_file", conversation_file_id)
            .await?
        else {
            return Err(BizError::new(
                DOCUMENT_TEXT_EXTRACTION_MANAGER_ERROR,
                "conversation file storage reference not found".to_string(),
            ));
        };

        let object = self
            .object_svc
            .get_by_sha256(&storage_ref.storage_object_sha256)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    DOCUMENT_TEXT_EXTRACTION_MANAGER_ERROR,
                    "storage object not found".to_string(),
                )
            })?;

        let analysis = self
            .analysis_api
            .ensure_pending(EnsureStorageObjectAnalysisRequest {
                storage_object_sha256: object.sha256.clone(),
                analysis_kind: DOCUMENT_TEXT_EXTRACTION_ANALYSIS_KIND.to_string(),
                prompt_key: DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY.to_string(),
                prompt_version: DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION.to_string(),
                llm_name: DOCUMENT_TEXT_EXTRACTION_LLM_NAME.to_string(),
                llm_model: None,
            })
            .await?;

        Ok((object, analysis))
    }
}

pub fn is_supported_document_text_file(file: &ConversationFile) -> bool {
    file.mime_type
        .as_deref()
        .is_some_and(is_supported_document_text_mime)
        || file
            .file_name
            .as_deref()
            .is_some_and(is_supported_document_text_name)
        || is_supported_document_text_name(&file.object_key)
}

pub fn is_supported_document_text_mime(value: &str) -> bool {
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        mime.as_str(),
        "application/pdf"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel"
            | "text/csv"
            | "text/plain"
            | "text/markdown"
            | "application/json"
    )
}

pub fn is_supported_document_text_name(value: &str) -> bool {
    let extension = value
        .rsplit_once('.')
        .map(|(_, extension)| extension.trim().to_ascii_lowercase());
    matches!(
        extension.as_deref(),
        Some(
            "pdf"
                | "docx"
                | "xlsx"
                | "xlsm"
                | "xlsb"
                | "xls"
                | "csv"
                | "txt"
                | "md"
                | "markdown"
                | "json"
        )
    )
}
