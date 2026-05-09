use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::workspace_artifacts,
    table::workspace_artifacts::dto::{
        CreateWorkspaceArtifact, UpdateWorkspaceArtifact, WorkspaceArtifact,
        WorkspaceArtifactFilter, WorkspaceArtifactStatus, WorkspaceArtifactValidationStatus,
    },
};

db_core::impl_repository!(
    WorkspaceArtifactRepo,
    workspace_artifacts::Entity,
    workspace_artifacts::Model
);

pub struct WorkspaceArtifactService {
    repo: WorkspaceArtifactRepo,
}

impl WorkspaceArtifactService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceArtifactRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateWorkspaceArtifact) -> BizResult<WorkspaceArtifact> {
        let model = workspace_artifacts::ActiveModel {
            id: Set(input.id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            workspace_task_id: Set(input.workspace_task_id),
            run_task_id: Set(input.run_task_id),
            user_id: Set(input.user_id),
            artifact_type: Set(input.artifact_type),
            title: Set(input.title),
            content: Set(input.content),
            preview: Set(input.preview),
            mime_type: Set(input.mime_type),
            format: Set(input.format),
            storage_kind: Set(input.storage_kind),
            bucket: Set(input.bucket),
            object_key: Set(input.object_key),
            file_name: Set(input.file_name),
            file_size: Set(input.file_size),
            sha256: Set(input.sha256),
            source: Set(input.source),
            status: Set(input.status.as_str().to_string()),
            validation_status: Set(input.validation_status.as_str().to_string()),
            validation_report: Set(input.validation_report),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<WorkspaceArtifact>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: WorkspaceArtifactFilter) -> BizResult<Vec<WorkspaceArtifact>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: WorkspaceArtifactFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateWorkspaceArtifact) -> BizResult<WorkspaceArtifact> {
        let model = workspace_artifacts::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            content: Set(input.content),
            preview: Set(input.preview),
            mime_type: Set(input.mime_type),
            format: Set(input.format),
            storage_kind: Set(input.storage_kind),
            bucket: Set(input.bucket),
            object_key: Set(input.object_key),
            file_name: Set(input.file_name),
            file_size: Set(input.file_size),
            sha256: Set(input.sha256),
            status: Set(input.status.as_str().to_string()),
            validation_status: Set(input.validation_status.as_str().to_string()),
            validation_report: Set(input.validation_report),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: WorkspaceArtifactFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<workspace_artifacts::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(workspace_artifacts::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(workspace_artifacts::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(workspace_artifacts::Column::RunId.eq(run_id));
        }

        if let Some(workspace_task_id) = filter.workspace_task_id {
            query =
                query.filter(workspace_artifacts::Column::WorkspaceTaskId.eq(workspace_task_id));
        }

        if let Some(run_task_id) = filter.run_task_id {
            query = query.filter(workspace_artifacts::Column::RunTaskId.eq(run_task_id));
        }

        if let Some(artifact_type) = filter.artifact_type {
            query = query.filter(workspace_artifacts::Column::ArtifactType.eq(artifact_type));
        }

        if let Some(status) = filter.status {
            query = query.filter(workspace_artifacts::Column::Status.eq(status.as_str()));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(workspace_artifacts::Column::Title.contains(keyword.clone()))
                .add(workspace_artifacts::Column::Preview.contains(keyword.clone()))
                .add(workspace_artifacts::Column::Content.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(workspace_artifacts::Column::CreatedAt);

        if include_pagination {
            if let Some(offset) = filter.offset {
                query = query.offset(offset);
            }

            if let Some(limit) = filter.limit {
                query = query.limit(limit);
            }
        }

        query
    }

    fn from_model(model: workspace_artifacts::Model) -> BizResult<WorkspaceArtifact> {
        Ok(WorkspaceArtifact {
            id: model.id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            workspace_task_id: model.workspace_task_id,
            run_task_id: model.run_task_id,
            user_id: model.user_id,
            artifact_type: model.artifact_type,
            title: model.title,
            content: model.content,
            preview: model.preview,
            mime_type: model.mime_type,
            format: model.format,
            storage_kind: model.storage_kind,
            bucket: model.bucket,
            object_key: model.object_key,
            file_name: model.file_name,
            file_size: model.file_size,
            sha256: model.sha256,
            source: model.source,
            status: WorkspaceArtifactStatus::try_from(model.status)?,
            validation_status: WorkspaceArtifactValidationStatus::try_from(
                model.validation_status.as_str(),
            )?,
            validation_report: model.validation_report,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
