use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::storage_object_analyses,
    table::storage_object_analyses::dto::{
        CreateStorageObjectAnalysis, StorageObjectAnalysis, StorageObjectAnalysisIdentity,
        StorageObjectAnalysisStatus, UpdateStorageObjectAnalysisProcessing,
        UpdateStorageObjectAnalysisSucceeded,
    },
};

db_core::impl_repository!(
    StorageObjectAnalysisRepo,
    storage_object_analyses::Entity,
    storage_object_analyses::Model
);

pub struct StorageObjectAnalysisService {
    repo: StorageObjectAnalysisRepo,
}

impl StorageObjectAnalysisService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StorageObjectAnalysisRepo::new(db),
        }
    }

    pub async fn create(
        &self,
        input: CreateStorageObjectAnalysis,
    ) -> BizResult<StorageObjectAnalysis> {
        let model = storage_object_analyses::ActiveModel {
            id: Set(input.id),
            storage_object_sha256: Set(input.storage_object_sha256),
            analysis_kind: Set(input.analysis_kind),
            status: Set(StorageObjectAnalysisStatus::Pending.as_str().to_string()),
            prompt_key: Set(input.prompt_key),
            prompt_version: Set(input.prompt_version),
            llm_name: Set(input.llm_name),
            llm_model: Set(input.llm_model),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<StorageObjectAnalysis>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn get_by_identity(
        &self,
        identity: &StorageObjectAnalysisIdentity,
    ) -> BizResult<Option<StorageObjectAnalysis>> {
        let query = self
            .repo
            .query()
            .filter(
                storage_object_analyses::Column::StorageObjectSha256
                    .eq(&identity.storage_object_sha256),
            )
            .filter(storage_object_analyses::Column::AnalysisKind.eq(&identity.analysis_kind))
            .filter(storage_object_analyses::Column::PromptKey.eq(&identity.prompt_key))
            .filter(storage_object_analyses::Column::PromptVersion.eq(&identity.prompt_version))
            .filter(storage_object_analyses::Column::LlmName.eq(&identity.llm_name));

        Ok(self.repo.select_one(query).await?.map(Self::from_model))
    }

    pub async fn latest_succeeded_for_object(
        &self,
        storage_object_sha256: &str,
        analysis_kind: &str,
    ) -> BizResult<Option<StorageObjectAnalysis>> {
        let query = self
            .repo
            .query()
            .filter(storage_object_analyses::Column::StorageObjectSha256.eq(storage_object_sha256))
            .filter(storage_object_analyses::Column::AnalysisKind.eq(analysis_kind))
            .filter(
                storage_object_analyses::Column::Status
                    .eq(StorageObjectAnalysisStatus::Succeeded.as_str()),
            )
            .order_by_desc(storage_object_analyses::Column::UpdatedAt)
            .limit(1);

        Ok(self.repo.select_one(query).await?.map(Self::from_model))
    }

    pub async fn mark_processing(
        &self,
        input: UpdateStorageObjectAnalysisProcessing,
    ) -> BizResult<StorageObjectAnalysis> {
        let model = storage_object_analyses::ActiveModel {
            id: Unchanged(input.id),
            status: Set(StorageObjectAnalysisStatus::Processing.as_str().to_string()),
            llm_model: Set(input.llm_model),
            started_at: Set(Some(input.started_at)),
            error_message: Set(None),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    pub async fn mark_succeeded(
        &self,
        input: UpdateStorageObjectAnalysisSucceeded,
    ) -> BizResult<StorageObjectAnalysis> {
        let model = storage_object_analyses::ActiveModel {
            id: Unchanged(input.id),
            status: Set(StorageObjectAnalysisStatus::Succeeded.as_str().to_string()),
            llm_model: Set(input.llm_model),
            summary: Set(Some(input.summary)),
            result_json: Set(Some(input.result_json)),
            error_message: Set(None),
            finished_at: Set(Some(input.finished_at)),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    pub async fn delete_by_id(&self, id: Uuid) -> BizResult<u64> {
        Ok(self.repo.delete_by_id(id).await?.rows_affected)
    }

    fn from_model(model: storage_object_analyses::Model) -> StorageObjectAnalysis {
        StorageObjectAnalysis {
            id: model.id,
            storage_object_sha256: model.storage_object_sha256,
            analysis_kind: model.analysis_kind,
            status: model.status,
            prompt_key: model.prompt_key,
            prompt_version: model.prompt_version,
            llm_name: model.llm_name,
            llm_model: model.llm_model,
            summary: model.summary,
            result_json: model.result_json,
            error_message: model.error_message,
            started_at: model.started_at,
            finished_at: model.finished_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
