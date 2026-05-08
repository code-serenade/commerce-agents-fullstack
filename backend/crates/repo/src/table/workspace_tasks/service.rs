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
    entity::workspace_tasks,
    table::workspace_tasks::dto::{
        CreateWorkspaceTask, UpdateWorkspaceTask, WorkspaceTask, WorkspaceTaskFilter,
        WorkspaceTaskStatus,
    },
};

db_core::impl_repository!(
    WorkspaceTaskRepo,
    workspace_tasks::Entity,
    workspace_tasks::Model
);

pub struct WorkspaceTaskService {
    repo: WorkspaceTaskRepo,
}

impl WorkspaceTaskService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceTaskRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateWorkspaceTask) -> BizResult<WorkspaceTask> {
        let model = workspace_tasks::ActiveModel {
            id: Set(input.id),
            workspace_id: Set(input.workspace_id),
            user_id: Set(input.user_id),
            title: Set(input.title),
            description: Set(input.description),
            owner_role: Set(input.owner_role),
            status: Set(input.status.as_str().to_string()),
            priority: Set(input.priority),
            depends_on: Set(input.depends_on),
            acceptance_criteria: Set(input.acceptance_criteria),
            result: Set(input.result),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<WorkspaceTask>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: WorkspaceTaskFilter) -> BizResult<Vec<WorkspaceTask>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: WorkspaceTaskFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateWorkspaceTask) -> BizResult<WorkspaceTask> {
        let model = workspace_tasks::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            description: Set(input.description),
            owner_role: Set(input.owner_role),
            status: Set(input.status.as_str().to_string()),
            priority: Set(input.priority),
            depends_on: Set(input.depends_on),
            acceptance_criteria: Set(input.acceptance_criteria),
            result: Set(input.result),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: WorkspaceTaskFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<workspace_tasks::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(workspace_tasks::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(workspace_tasks::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(workspace_tasks::Column::Status.eq(status.as_str()));
        }

        if let Some(owner_role) = filter.owner_role {
            query = query.filter(workspace_tasks::Column::OwnerRole.eq(owner_role));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(workspace_tasks::Column::Title.contains(keyword.clone()))
                .add(workspace_tasks::Column::Description.contains(keyword.clone()))
                .add(workspace_tasks::Column::AcceptanceCriteria.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query
            .order_by_desc(workspace_tasks::Column::Priority)
            .order_by_asc(workspace_tasks::Column::CreatedAt);

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

    fn from_model(model: workspace_tasks::Model) -> BizResult<WorkspaceTask> {
        Ok(WorkspaceTask {
            id: model.id,
            workspace_id: model.workspace_id,
            user_id: model.user_id,
            title: model.title,
            description: model.description,
            owner_role: model.owner_role,
            status: WorkspaceTaskStatus::try_from(model.status.as_str())?,
            priority: model.priority,
            depends_on: model.depends_on,
            acceptance_criteria: model.acceptance_criteria,
            result: model.result,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
