use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::long_running_tasks,
    table::long_running_tasks::dto::{
        CreateLongRunningTask, LongRunningTask, LongRunningTaskFilter, LongTaskStatus,
        UpdateLongRunningTask,
    },
};

db_core::impl_repository!(
    LongRunningTaskRepo,
    long_running_tasks::Entity,
    long_running_tasks::Model
);

pub struct LongRunningTaskService {
    repo: LongRunningTaskRepo,
}

impl LongRunningTaskService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: LongRunningTaskRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateLongRunningTask) -> BizResult<LongRunningTask> {
        let model = long_running_tasks::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            task_type: Set(input.task_type),
            status: Set(input.status.as_str().to_string()),
            title: Set(input.title),
            progress: Set(input.progress),
            total_steps: Set(input.total_steps),
            current_step: Set(input.current_step),
            role: Set(input.role),
            input_params: Set(input.input_params),
            result: Set(input.result),
            error_message: Set(input.error_message),
            started_at: Set(input.started_at),
            completed_at: Set(input.completed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<LongRunningTask>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: LongRunningTaskFilter) -> BizResult<Vec<LongRunningTask>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: LongRunningTaskFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateLongRunningTask) -> BizResult<LongRunningTask> {
        let model = long_running_tasks::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            title: Set(input.title),
            progress: Set(input.progress),
            total_steps: Set(input.total_steps),
            current_step: Set(input.current_step),
            role: Set(input.role),
            result: Set(input.result),
            error_message: Set(input.error_message),
            started_at: Set(input.started_at),
            completed_at: Set(input.completed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: LongRunningTaskFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<long_running_tasks::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(long_running_tasks::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(long_running_tasks::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(long_running_tasks::Column::RunId.eq(run_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(long_running_tasks::Column::Status.eq(status.as_str()));
        }

        if let Some(task_type) = filter.task_type {
            query = query.filter(long_running_tasks::Column::TaskType.eq(task_type));
        }

        query = query.order_by_desc(long_running_tasks::Column::CreatedAt);

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

    fn from_model(model: long_running_tasks::Model) -> BizResult<LongRunningTask> {
        Ok(LongRunningTask {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            task_type: model.task_type,
            status: LongTaskStatus::try_from(model.status.as_str())?,
            title: model.title,
            progress: model.progress,
            total_steps: model.total_steps,
            current_step: model.current_step,
            role: model.role,
            input_params: model.input_params,
            result: model.result,
            error_message: model.error_message,
            started_at: model.started_at,
            completed_at: model.completed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
