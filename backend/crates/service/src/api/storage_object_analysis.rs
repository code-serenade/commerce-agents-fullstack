use db_core::{DbContext, error::BizResult};
use repo::table::storage_object_analyses::{
    CreateStorageObjectAnalysis, StorageObjectAnalysis, StorageObjectAnalysisIdentity,
    StorageObjectAnalysisService, StorageObjectAnalysisStatus, UpdateStorageObjectAnalysisFailed,
    UpdateStorageObjectAnalysisProcessing, UpdateStorageObjectAnalysisSucceeded,
};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct StorageObjectAnalysisApi {
    analysis_svc: StorageObjectAnalysisService,
}

#[derive(Debug, Clone)]
pub struct EnsureStorageObjectAnalysisRequest {
    pub storage_object_sha256: String,
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompleteStorageObjectAnalysisRequest {
    pub id: Uuid,
    pub llm_model: Option<String>,
    pub summary: String,
    pub result_json: Value,
}

#[derive(Debug, Clone)]
pub struct FailStorageObjectAnalysisRequest {
    pub id: Uuid,
    pub llm_model: Option<String>,
    pub error_message: String,
}

impl StorageObjectAnalysisApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            analysis_svc: StorageObjectAnalysisService::new(db),
        }
    }

    pub async fn ensure_pending(
        &self,
        req: EnsureStorageObjectAnalysisRequest,
    ) -> BizResult<StorageObjectAnalysis> {
        let identity = StorageObjectAnalysisIdentity {
            storage_object_sha256: req.storage_object_sha256,
            analysis_kind: req.analysis_kind,
            prompt_key: req.prompt_key,
            prompt_version: req.prompt_version,
            llm_name: req.llm_name,
        };

        if let Some(existing) = self.analysis_svc.get_by_identity(&identity).await? {
            return Ok(existing);
        }

        self.analysis_svc
            .create(CreateStorageObjectAnalysis {
                id: Uuid::new_v4(),
                storage_object_sha256: identity.storage_object_sha256,
                analysis_kind: identity.analysis_kind,
                prompt_key: identity.prompt_key,
                prompt_version: identity.prompt_version,
                llm_name: identity.llm_name,
                llm_model: req.llm_model,
            })
            .await
    }

    pub async fn latest_succeeded_for_object(
        &self,
        storage_object_sha256: &str,
        analysis_kind: &str,
    ) -> BizResult<Option<StorageObjectAnalysis>> {
        self.analysis_svc
            .latest_succeeded_for_object(storage_object_sha256, analysis_kind)
            .await
    }

    pub async fn mark_processing(&self, id: Uuid, llm_model: Option<String>) -> BizResult<()> {
        self.analysis_svc
            .mark_processing(UpdateStorageObjectAnalysisProcessing {
                id,
                llm_model,
                started_at: OffsetDateTime::now_utc(),
            })
            .await?;
        Ok(())
    }

    pub async fn mark_succeeded(&self, req: CompleteStorageObjectAnalysisRequest) -> BizResult<()> {
        self.analysis_svc
            .mark_succeeded(UpdateStorageObjectAnalysisSucceeded {
                id: req.id,
                llm_model: req.llm_model,
                summary: req.summary,
                result_json: req.result_json,
                finished_at: OffsetDateTime::now_utc(),
            })
            .await?;
        Ok(())
    }

    pub async fn mark_failed(&self, req: FailStorageObjectAnalysisRequest) -> BizResult<()> {
        self.analysis_svc
            .mark_failed(UpdateStorageObjectAnalysisFailed {
                id: req.id,
                llm_model: req.llm_model,
                error_message: req.error_message,
                finished_at: OffsetDateTime::now_utc(),
            })
            .await?;
        Ok(())
    }

    pub fn can_process(status: &str) -> bool {
        status == StorageObjectAnalysisStatus::Pending.as_str()
    }
}
