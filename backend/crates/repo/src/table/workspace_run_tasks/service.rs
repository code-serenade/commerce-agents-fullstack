use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::workspace_run_tasks,
    table::workspace_run_tasks::dto::{
        CreateWorkspaceRunTask, UpdateWorkspaceRunTask, WorkspaceRunTask, WorkspaceRunTaskFilter,
        WorkspaceRunTaskStatus,
    },
};

db_core::impl_repository!(
    WorkspaceRunTaskRepo,
    workspace_run_tasks::Entity,
    workspace_run_tasks::Model
);

pub struct WorkspaceRunTaskService {
    repo: WorkspaceRunTaskRepo,
}

impl WorkspaceRunTaskService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceRunTaskRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateWorkspaceRunTask) -> BizResult<WorkspaceRunTask> {
        let model = workspace_run_tasks::ActiveModel {
            id: Set(input.id),
            run_id: Set(input.run_id),
            workspace_id: Set(input.workspace_id),
            workspace_task_id: Set(input.workspace_task_id),
            user_id: Set(input.user_id),
            seq: Set(input.seq),
            owner_role: Set(input.owner_role),
            status: Set(input.status.as_str().to_string()),
            attempts: Set(input.attempts),
            acceptance_score: Set(input.acceptance_score),
            acceptance_passed: Set(input.acceptance_passed),
            acceptance_notes: Set(input.acceptance_notes),
            execution_notes: Set(input.execution_notes),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<WorkspaceRunTask>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: WorkspaceRunTaskFilter) -> BizResult<Vec<WorkspaceRunTask>> {
        let mut query = self
            .repo
            .query()
            .filter(workspace_run_tasks::Column::UserId.eq(filter.user_id));

        if let Some(run_id) = filter.run_id {
            query = query.filter(workspace_run_tasks::Column::RunId.eq(run_id));
        }

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(workspace_run_tasks::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(workspace_run_tasks::Column::Status.eq(status.as_str()));
        }

        query = query.order_by_asc(workspace_run_tasks::Column::Seq);

        if let Some(offset) = filter.offset {
            query = query.offset(offset);
        }

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn update(&self, input: UpdateWorkspaceRunTask) -> BizResult<WorkspaceRunTask> {
        let model = workspace_run_tasks::ActiveModel {
            id: Unchanged(input.id),
            seq: Set(input.seq),
            owner_role: Set(input.owner_role),
            status: Set(input.status.as_str().to_string()),
            attempts: Set(input.attempts),
            acceptance_score: Set(input.acceptance_score),
            acceptance_passed: Set(input.acceptance_passed),
            acceptance_notes: Set(input.acceptance_notes),
            execution_notes: Set(input.execution_notes),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: workspace_run_tasks::Model) -> BizResult<WorkspaceRunTask> {
        Ok(WorkspaceRunTask {
            id: model.id,
            run_id: model.run_id,
            workspace_id: model.workspace_id,
            workspace_task_id: model.workspace_task_id,
            user_id: model.user_id,
            seq: model.seq,
            owner_role: model.owner_role,
            status: WorkspaceRunTaskStatus::try_from(model.status.as_str())?,
            attempts: model.attempts,
            acceptance_score: model.acceptance_score,
            acceptance_passed: model.acceptance_passed,
            acceptance_notes: model.acceptance_notes,
            execution_notes: model.execution_notes,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
