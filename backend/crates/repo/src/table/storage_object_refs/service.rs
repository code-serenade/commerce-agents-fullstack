use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    entity::storage_object_refs,
    table::storage_object_refs::dto::{
        CreateStorageObjectRef, StorageObjectRef, StorageObjectRefStatus,
        UpdateStorageObjectRefStatus,
    },
};

db_core::impl_repository!(
    StorageObjectRefRepo,
    storage_object_refs::Entity,
    storage_object_refs::Model
);

pub struct StorageObjectRefService {
    repo: StorageObjectRefRepo,
}

impl StorageObjectRefService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StorageObjectRefRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateStorageObjectRef) -> BizResult<StorageObjectRef> {
        let model = storage_object_refs::ActiveModel {
            id: Set(input.id),
            storage_object_id: Set(input.storage_object_id),
            user_id: Set(input.user_id),
            source_type: Set(input.source_type),
            source_id: Set(input.source_id),
            status: Set(StorageObjectRefStatus::Active.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_active_by_source(
        &self,
        source_type: &str,
        source_id: Uuid,
    ) -> BizResult<Option<StorageObjectRef>> {
        let query = self
            .repo
            .query()
            .filter(storage_object_refs::Column::SourceType.eq(source_type))
            .filter(storage_object_refs::Column::SourceId.eq(source_id))
            .filter(
                storage_object_refs::Column::Status.eq(StorageObjectRefStatus::Active.as_str()),
            );

        Ok(self.repo.select_one(query).await?.map(Self::from_model))
    }

    pub async fn update_status(
        &self,
        input: UpdateStorageObjectRefStatus,
    ) -> BizResult<StorageObjectRef> {
        let model = storage_object_refs::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            released_at: Set(input.released_at),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    pub async fn count_active_by_object(&self, storage_object_id: Uuid) -> BizResult<u64> {
        let count = storage_object_refs::Entity::find()
            .filter(storage_object_refs::Column::StorageObjectId.eq(storage_object_id))
            .filter(storage_object_refs::Column::Status.eq(StorageObjectRefStatus::Active.as_str()))
            .count(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(count)
    }

    fn from_model(model: storage_object_refs::Model) -> StorageObjectRef {
        StorageObjectRef {
            id: model.id,
            storage_object_id: model.storage_object_id,
            user_id: model.user_id,
            source_type: model.source_type,
            source_id: model.source_id,
            status: model.status,
            released_at: model.released_at,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
