use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use repo::table::{
    storage_object_refs::StorageObjectRefService,
    storage_objects::{
        StorageObject, StorageObjectCleanupFilter, StorageObjectService, StorageObjectStatus,
        UpdateStorageObjectReferenceState,
    },
};
use storage::{BucketKind, DeleteObjectRequest, ObjectStorage};
use time::OffsetDateTime;

const STORAGE_RESOURCE_MANAGER_ERROR: i16 = -1;
const DEFAULT_CLEANUP_LIMIT: u64 = 100;

pub struct StorageResourceManager {
    object_svc: StorageObjectService,
    ref_svc: StorageObjectRefService,
}

#[derive(Debug, Clone)]
pub struct CleanupOrphanedStorageObjectsRequest {
    pub limit: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CleanupOrphanedStorageObjectsResponse {
    pub scanned: usize,
    pub deleted: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl StorageResourceManager {
    pub fn new(db: DbContext) -> Self {
        Self {
            object_svc: StorageObjectService::new(db.clone()),
            ref_svc: StorageObjectRefService::new(db),
        }
    }

    pub async fn cleanup_orphaned_objects(
        &self,
        storage: &ObjectStorage,
        req: CleanupOrphanedStorageObjectsRequest,
    ) -> BizResult<CleanupOrphanedStorageObjectsResponse> {
        let limit = req.limit.unwrap_or(DEFAULT_CLEANUP_LIMIT);
        let objects = self
            .object_svc
            .list_orphaned_for_cleanup(StorageObjectCleanupFilter { limit: Some(limit) })
            .await?;
        let scanned = objects.len();
        let mut deleted = 0;
        let mut skipped = 0;
        let mut failed = 0;

        for object in objects {
            match self.delete_if_unreferenced(storage, object).await {
                Ok(true) => deleted += 1,
                Ok(false) => skipped += 1,
                Err(_) => failed += 1,
            }
        }

        Ok(CleanupOrphanedStorageObjectsResponse {
            scanned,
            deleted,
            skipped,
            failed,
        })
    }

    pub async fn delete_if_unreferenced(
        &self,
        storage: &ObjectStorage,
        object: StorageObject,
    ) -> BizResult<bool> {
        let active_refs = self.ref_svc.count_active_by_object(object.id).await?;
        if active_refs > 0 {
            self.object_svc
                .update_reference_state(UpdateStorageObjectReferenceState {
                    id: object.id,
                    status: StorageObjectStatus::Active,
                    ref_count: active_refs as i64,
                    last_referenced_at: Some(OffsetDateTime::now_utc()),
                    orphaned_at: None,
                    deleted_at: None,
                })
                .await?;
            return Ok(false);
        }

        let bucket = bucket_kind_from_name(&object.bucket, storage)?;
        match storage
            .delete_object(DeleteObjectRequest {
                bucket,
                key: object.object_key.clone(),
            })
            .await
        {
            Ok(_) => {}
            Err(err) if is_not_found_storage_error(&err) => {}
            Err(err) => {
                return Err(BizError::new(
                    STORAGE_RESOURCE_MANAGER_ERROR,
                    err.to_string(),
                ));
            }
        }

        self.object_svc
            .update_reference_state(UpdateStorageObjectReferenceState {
                id: object.id,
                status: StorageObjectStatus::Deleted,
                ref_count: 0,
                last_referenced_at: object.last_referenced_at,
                orphaned_at: object.orphaned_at,
                deleted_at: Some(OffsetDateTime::now_utc()),
            })
            .await?;

        Ok(true)
    }
}

fn bucket_kind_from_name(bucket: &str, storage: &ObjectStorage) -> BizResult<BucketKind> {
    let config = storage.config();
    if bucket == "private" || bucket == config.private_bucket {
        Ok(BucketKind::Private)
    } else if bucket == "public" || bucket == config.public_bucket {
        Ok(BucketKind::Public)
    } else {
        Err(BizError::new(
            STORAGE_RESOURCE_MANAGER_ERROR,
            "invalid storage object bucket".to_string(),
        ))
    }
}

fn is_not_found_storage_error(err: &storage::Error) -> bool {
    err.to_string().contains("s3 error 404")
}
