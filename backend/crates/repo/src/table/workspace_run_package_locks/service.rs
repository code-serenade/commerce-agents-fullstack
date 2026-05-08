use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    entity::workspace_run_package_locks,
    table::{
        package_registry::{PackageStatus, PackageType},
        workspace_run_package_locks::dto::{
            CreateWorkspaceRunPackageLock, WorkspaceRunPackageLock, WorkspaceRunPackageLockFilter,
        },
    },
};

db_core::impl_repository!(
    WorkspaceRunPackageLockRepo,
    workspace_run_package_locks::Entity,
    workspace_run_package_locks::Model
);

pub struct WorkspaceRunPackageLockService {
    repo: WorkspaceRunPackageLockRepo,
}

impl WorkspaceRunPackageLockService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceRunPackageLockRepo::new(db),
        }
    }

    pub async fn create(
        &self,
        input: CreateWorkspaceRunPackageLock,
    ) -> BizResult<WorkspaceRunPackageLock> {
        let model = workspace_run_package_locks::ActiveModel {
            id: Set(input.id),
            run_id: Set(input.run_id),
            workspace_id: Set(input.workspace_id),
            user_id: Set(input.user_id),
            package_registry_id: Set(input.package_registry_id),
            package_type: Set(input.package_type.as_str().to_string()),
            package_id: Set(input.package_id),
            package_name: Set(input.package_name),
            locked_version: Set(input.locked_version),
            locked_status: Set(input.locked_status.as_str().to_string()),
            required: Set(input.required),
            lock_source: Set(input.lock_source),
            manifest_path: Set(input.manifest_path),
            notes: Set(input.notes),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list(
        &self,
        filter: WorkspaceRunPackageLockFilter,
    ) -> BizResult<Vec<WorkspaceRunPackageLock>> {
        self.repo
            .select_all(
                self.repo
                    .query()
                    .filter(workspace_run_package_locks::Column::UserId.eq(filter.user_id))
                    .filter(
                        workspace_run_package_locks::Column::WorkspaceId.eq(filter.workspace_id),
                    )
                    .filter(workspace_run_package_locks::Column::RunId.eq(filter.run_id))
                    .order_by_asc(workspace_run_package_locks::Column::CreatedAt),
            )
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn replace_for_run(
        &self,
        run_id: Uuid,
        entries: Vec<CreateWorkspaceRunPackageLock>,
    ) -> BizResult<Vec<WorkspaceRunPackageLock>> {
        workspace_run_package_locks::Entity::delete_many()
            .filter(workspace_run_package_locks::Column::RunId.eq(run_id))
            .exec(self.repo.db())
            .await
            .map_err(Error::from)?;

        let mut locks = Vec::with_capacity(entries.len());
        for entry in entries {
            locks.push(self.create(entry).await?);
        }
        Ok(locks)
    }

    fn from_model(model: workspace_run_package_locks::Model) -> BizResult<WorkspaceRunPackageLock> {
        Ok(WorkspaceRunPackageLock {
            id: model.id,
            run_id: model.run_id,
            workspace_id: model.workspace_id,
            user_id: model.user_id,
            package_registry_id: model.package_registry_id,
            package_type: PackageType::try_from(model.package_type.as_str())?,
            package_id: model.package_id,
            package_name: model.package_name,
            locked_version: model.locked_version,
            locked_status: PackageStatus::try_from(model.locked_status.as_str())?,
            required: model.required,
            lock_source: model.lock_source,
            manifest_path: model.manifest_path,
            notes: model.notes,
            metadata: model.metadata,
            created_at: model.created_at,
        })
    }
}
