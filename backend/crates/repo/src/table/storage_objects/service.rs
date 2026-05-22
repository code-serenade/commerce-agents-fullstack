use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::ActiveValue::{Set, Unchanged};
use serde_json::Value;

use crate::{
    entity::storage_objects,
    table::storage_objects::dto::{CreateStorageObject, StorageObject},
};

db_core::impl_repository!(
    StorageObjectRepo,
    storage_objects::Entity,
    storage_objects::Model
);

pub struct StorageObjectService {
    repo: StorageObjectRepo,
}

impl StorageObjectService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StorageObjectRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateStorageObject) -> BizResult<StorageObject> {
        let model = storage_objects::ActiveModel {
            sha256: Set(input.sha256),
            mime_type: Set(input.mime_type),
            file_size: Set(input.file_size),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_sha256(&self, sha256: &str) -> BizResult<Option<StorageObject>> {
        Ok(self
            .repo
            .find_by_id(sha256.to_string())
            .await?
            .map(Self::from_model))
    }

    pub async fn merge_metadata(
        &self,
        sha256: String,
        mime_type: Option<String>,
        file_size: Option<i64>,
        metadata: Value,
    ) -> BizResult<StorageObject> {
        let model = storage_objects::ActiveModel {
            sha256: Unchanged(sha256),
            mime_type: Set(mime_type),
            file_size: Set(file_size),
            metadata: Set(metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    fn from_model(model: storage_objects::Model) -> StorageObject {
        StorageObject {
            sha256: model.sha256,
            mime_type: model.mime_type,
            file_size: model.file_size,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
