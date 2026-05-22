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

const VISION_PREPROCESS_MANAGER_ERROR: i16 = -1;
pub const IMAGE_PREPROCESS_ANALYSIS_KIND: &str = "image_preprocess";

#[derive(Debug, Clone)]
pub struct VisionPreprocessConfig {
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnsureVisionPreprocessRequest {
    pub conversation_file_id: Uuid,
    pub config: VisionPreprocessConfig,
}

#[derive(Debug, Clone)]
pub struct VisionPreprocessTask {
    pub file: ConversationFile,
    pub object: StorageObject,
    pub analysis: StorageObjectAnalysis,
}

pub struct VisionPreprocessManager {
    file_svc: ConversationFileService,
    ref_svc: StorageObjectRefService,
    object_svc: StorageObjectService,
    analysis_api: StorageObjectAnalysisApi,
}

impl VisionPreprocessManager {
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
        req: EnsureVisionPreprocessRequest,
    ) -> BizResult<Option<VisionPreprocessTask>> {
        let Some(file) = self.file_svc.get_by_id(req.conversation_file_id).await? else {
            return Err(BizError::new(
                VISION_PREPROCESS_MANAGER_ERROR,
                "conversation file not found".to_string(),
            ));
        };
        if file.status != "active" || !is_supported_image_mime(file.mime_type.as_deref()) {
            return Ok(None);
        }

        let Some(storage_ref) = self
            .ref_svc
            .get_active_by_source("conversation_file", file.id)
            .await?
        else {
            return Err(BizError::new(
                VISION_PREPROCESS_MANAGER_ERROR,
                "conversation file storage reference not found".to_string(),
            ));
        };

        let object = self
            .object_svc
            .get_by_sha256(&storage_ref.storage_object_sha256)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    VISION_PREPROCESS_MANAGER_ERROR,
                    "storage object not found".to_string(),
                )
            })?;

        let analysis = self
            .analysis_api
            .ensure_pending(EnsureStorageObjectAnalysisRequest {
                storage_object_sha256: object.sha256.clone(),
                analysis_kind: req.config.analysis_kind,
                prompt_key: req.config.prompt_key,
                prompt_version: req.config.prompt_version,
                llm_name: req.config.llm_name,
                llm_model: req.config.llm_model,
            })
            .await?;

        Ok(Some(VisionPreprocessTask {
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
                IMAGE_PREPROCESS_ANALYSIS_KIND,
            )
            .await
    }
}

pub fn is_supported_image_mime(mime_type: Option<&str>) -> bool {
    matches!(
        mime_type.map(|value| value.trim().to_ascii_lowercase()),
        Some(value)
            if matches!(
                value.as_str(),
                "image/png" | "image/jpeg" | "image/jpg" | "image/webp"
            )
    )
}
