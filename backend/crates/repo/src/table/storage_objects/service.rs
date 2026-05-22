use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::storage_objects,
    table::storage_objects::dto::{
        CreateStorageObject, StorageObject, StorageObjectCleanupFilter, StorageObjectStatus,
        UpdateStorageObjectReferenceState,
    },
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
            id: Set(input.id),
            user_id: Set(input.user_id),
            bucket: Set(input.bucket),
            object_key: Set(input.object_key),
            sha256: Set(input.sha256),
            mime_type: Set(input.mime_type),
            file_size: Set(input.file_size),
            storage_provider: Set("s3".to_string()),
            status: Set("active".to_string()),
            ref_count: Set(0),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<StorageObject>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn get_by_user_object(
        &self,
        user_id: Uuid,
        bucket: &str,
        object_key: &str,
    ) -> BizResult<Option<StorageObject>> {
        let query = self
            .repo
            .query()
            .filter(storage_objects::Column::UserId.eq(user_id))
            .filter(storage_objects::Column::Bucket.eq(bucket))
            .filter(storage_objects::Column::ObjectKey.eq(object_key));

        Ok(self.repo.select_one(query).await?.map(Self::from_model))
    }

    pub async fn update_reference_state(
        &self,
        input: UpdateStorageObjectReferenceState,
    ) -> BizResult<StorageObject> {
        let model = storage_objects::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            ref_count: Set(input.ref_count),
            last_referenced_at: Set(input.last_referenced_at),
            orphaned_at: Set(input.orphaned_at),
            deleted_at: Set(input.deleted_at),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    pub async fn list_orphaned_for_cleanup(
        &self,
        filter: StorageObjectCleanupFilter,
    ) -> BizResult<Vec<StorageObject>> {
        let mut query = self
            .repo
            .query()
            .filter(storage_objects::Column::Status.eq(StorageObjectStatus::Orphaned.as_str()))
            .order_by_asc(storage_objects::Column::OrphanedAt);

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    fn from_model(model: storage_objects::Model) -> StorageObject {
        StorageObject {
            id: model.id,
            user_id: model.user_id,
            bucket: model.bucket,
            object_key: model.object_key,
            sha256: model.sha256,
            mime_type: model.mime_type,
            file_size: model.file_size,
            storage_provider: model.storage_provider,
            status: model.status,
            ref_count: model.ref_count,
            last_referenced_at: model.last_referenced_at,
            orphaned_at: model.orphaned_at,
            deleted_at: model.deleted_at,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
