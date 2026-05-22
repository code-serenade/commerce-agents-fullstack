use db_core::{DbContext, error::BizResult};
use repo::table::storage_objects::{StorageObject, StorageObjectService};
use storage::ObjectStorage;

pub struct StorageResourceManager {
    _object_svc: StorageObjectService,
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
            _object_svc: StorageObjectService::new(db),
        }
    }

    pub async fn cleanup_orphaned_objects(
        &self,
        _storage: &ObjectStorage,
        _req: CleanupOrphanedStorageObjectsRequest,
    ) -> BizResult<CleanupOrphanedStorageObjectsResponse> {
        Ok(CleanupOrphanedStorageObjectsResponse {
            scanned: 0,
            deleted: 0,
            skipped: 0,
            failed: 0,
        })
    }

    pub async fn delete_if_unreferenced(
        &self,
        _storage: &ObjectStorage,
        _object: StorageObject,
    ) -> BizResult<bool> {
        Ok(false)
    }
}
