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
    entity::workspace_runs,
    table::workspace_runs::dto::{
        CreateWorkspaceRun, UpdateWorkspaceRun, WorkspaceRun, WorkspaceRunFilter,
        WorkspaceRunStatus,
    },
};

db_core::impl_repository!(
    WorkspaceRunRepo,
    workspace_runs::Entity,
    workspace_runs::Model
);

pub struct WorkspaceRunService {
    repo: WorkspaceRunRepo,
}

impl WorkspaceRunService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceRunRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateWorkspaceRun) -> BizResult<WorkspaceRun> {
        let model = workspace_runs::ActiveModel {
            id: Set(input.id),
            workspace_id: Set(input.workspace_id),
            user_id: Set(input.user_id),
            goal: Set(input.goal),
            requirements: Set(input.requirements),
            status: Set(input.status.as_str().to_string()),
            source: Set(input.source),
            summary: Set(input.summary),
            report: Set(input.report),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<WorkspaceRun>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: WorkspaceRunFilter) -> BizResult<Vec<WorkspaceRun>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: WorkspaceRunFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateWorkspaceRun) -> BizResult<WorkspaceRun> {
        let model = workspace_runs::ActiveModel {
            id: Unchanged(input.id),
            goal: Set(input.goal),
            requirements: Set(input.requirements),
            status: Set(input.status.as_str().to_string()),
            source: Set(input.source),
            summary: Set(input.summary),
            report: Set(input.report),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: WorkspaceRunFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<workspace_runs::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(workspace_runs::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(workspace_runs::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(workspace_runs::Column::Status.eq(status.as_str()));
        }

        if let Some(source) = filter.source {
            query = query.filter(workspace_runs::Column::Source.eq(source));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(workspace_runs::Column::Goal.contains(keyword.clone()))
                .add(workspace_runs::Column::Summary.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(workspace_runs::Column::UpdatedAt);

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

    fn from_model(model: workspace_runs::Model) -> BizResult<WorkspaceRun> {
        Ok(WorkspaceRun {
            id: model.id,
            workspace_id: model.workspace_id,
            user_id: model.user_id,
            goal: model.goal,
            requirements: model.requirements,
            status: WorkspaceRunStatus::try_from(model.status.as_str())?,
            source: model.source,
            summary: model.summary,
            report: model.report,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
