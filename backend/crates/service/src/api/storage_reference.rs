use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use repo::table::{
    storage_object_refs::{
        CreateStorageObjectRef, StorageObjectRef, StorageObjectRefService, StorageObjectRefStatus,
        UpdateStorageObjectRefStatus,
    },
    storage_objects::{
        CreateStorageObject, StorageObject, StorageObjectService, StorageObjectStatus,
        UpdateStorageObjectReferenceState,
    },
};
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

const STORAGE_REFERENCE_ERROR: i16 = -1;

pub struct StorageReferenceApi {
    db: DbContext,
    object_svc: StorageObjectService,
    ref_svc: StorageObjectRefService,
}

pub struct RetainStorageReferenceRequest {
    pub user_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub sha256: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub source_type: String,
    pub source_id: Uuid,
    pub metadata: Value,
}

pub struct ReleaseStorageReferenceRequest {
    pub source_type: String,
    pub source_id: Uuid,
}

impl StorageReferenceApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            object_svc: StorageObjectService::new(db.clone()),
            ref_svc: StorageObjectRefService::new(db),
        }
    }

    pub async fn retain(&self, req: RetainStorageReferenceRequest) -> BizResult<StorageObjectRef> {
        let tx = self.db.begin().await?;
        let result: BizResult<StorageObjectRef> = async {
            let api = StorageReferenceApi::new(tx.clone());
            api.retain_in_current_context(req).await
        }
        .await;
        commit_or_rollback(tx, result).await
    }

    pub async fn release(&self, req: ReleaseStorageReferenceRequest) -> BizResult<()> {
        let tx = self.db.begin().await?;
        let result: BizResult<()> = async {
            let api = StorageReferenceApi::new(tx.clone());
            api.release_in_current_context(&req.source_type, req.source_id)
                .await
        }
        .await;
        commit_or_rollback(tx, result).await
    }

    pub async fn retain_in_current_context(
        &self,
        req: RetainStorageReferenceRequest,
    ) -> BizResult<StorageObjectRef> {
        let now = OffsetDateTime::now_utc();
        let object = self.find_or_create_object(&req, now).await?;

        if let Some(existing_ref) = self
            .ref_svc
            .get_active_by_source(&req.source_type, req.source_id)
            .await?
        {
            if existing_ref.storage_object_id == object.id {
                self.refresh_object_ref_count(object, now).await?;
                return Ok(existing_ref);
            }

            self.release_active_ref(existing_ref, now).await?;
        }

        let storage_ref = self
            .ref_svc
            .create(CreateStorageObjectRef {
                id: Uuid::new_v4(),
                storage_object_id: object.id,
                user_id: req.user_id,
                source_type: req.source_type,
                source_id: req.source_id,
                metadata: req.metadata,
            })
            .await?;
        self.refresh_object_ref_count(object, now).await?;

        Ok(storage_ref)
    }

    pub async fn release_in_current_context(
        &self,
        source_type: &str,
        source_id: Uuid,
    ) -> BizResult<()> {
        let Some(storage_ref) = self
            .ref_svc
            .get_active_by_source(source_type, source_id)
            .await?
        else {
            return Ok(());
        };

        self.release_active_ref(storage_ref, OffsetDateTime::now_utc())
            .await
    }

    async fn find_or_create_object(
        &self,
        req: &RetainStorageReferenceRequest,
        now: OffsetDateTime,
    ) -> BizResult<StorageObject> {
        if let Some(object) = self
            .object_svc
            .get_by_user_object(req.user_id, &req.bucket, &req.object_key)
            .await?
        {
            return Ok(object);
        }

        let object = self
            .object_svc
            .create(CreateStorageObject {
                id: Uuid::new_v4(),
                user_id: req.user_id,
                bucket: req.bucket.clone(),
                object_key: req.object_key.clone(),
                sha256: req.sha256.clone(),
                mime_type: req.mime_type.clone(),
                file_size: req.file_size,
                metadata: json!({
                    "createdFrom": {
                        "sourceType": req.source_type,
                        "sourceId": req.source_id,
                        "retainedAt": now,
                    }
                }),
            })
            .await?;

        Ok(object)
    }

    async fn release_active_ref(
        &self,
        storage_ref: StorageObjectRef,
        now: OffsetDateTime,
    ) -> BizResult<()> {
        self.ref_svc
            .update_status(UpdateStorageObjectRefStatus {
                id: storage_ref.id,
                status: StorageObjectRefStatus::Released,
                released_at: Some(now),
            })
            .await?;

        let object = self
            .object_svc
            .get_by_id(storage_ref.storage_object_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    STORAGE_REFERENCE_ERROR,
                    "storage object not found for active reference".to_string(),
                )
            })?;
        self.refresh_object_ref_count(object, now).await?;

        Ok(())
    }

    async fn refresh_object_ref_count(
        &self,
        object: StorageObject,
        now: OffsetDateTime,
    ) -> BizResult<StorageObject> {
        let ref_count = self.ref_svc.count_active_by_object(object.id).await? as i64;
        let (status, orphaned_at) = if ref_count == 0 {
            (
                StorageObjectStatus::Orphaned,
                object.orphaned_at.or(Some(now)),
            )
        } else {
            (StorageObjectStatus::Active, None)
        };

        self.object_svc
            .update_reference_state(UpdateStorageObjectReferenceState {
                id: object.id,
                status,
                ref_count,
                last_referenced_at: if ref_count > 0 {
                    Some(now)
                } else {
                    object.last_referenced_at
                },
                orphaned_at,
                deleted_at: None,
            })
            .await
    }
}

async fn commit_or_rollback<T>(tx: DbContext, result: BizResult<T>) -> BizResult<T> {
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
