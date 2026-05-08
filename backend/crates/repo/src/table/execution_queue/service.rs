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
    entity::execution_queue,
    table::execution_queue::dto::{
        CreateExecutionQueueItem, ExecutionQueueFilter, ExecutionQueueItem, ExecutionQueueStatus,
        UpdateExecutionQueueItem,
    },
};

db_core::impl_repository!(
    ExecutionQueueRepo,
    execution_queue::Entity,
    execution_queue::Model
);

pub struct ExecutionQueueService {
    repo: ExecutionQueueRepo,
}

impl ExecutionQueueService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ExecutionQueueRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateExecutionQueueItem) -> BizResult<ExecutionQueueItem> {
        let model = execution_queue::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            workspace_task_id: Set(input.workspace_task_id),
            action_type: Set(input.action_type),
            description: Set(input.description),
            platform: Set(input.platform),
            payload: Set(input.payload),
            expected_impact: Set(input.expected_impact),
            status: Set(input.status.as_str().to_string()),
            decision_reason: Set(input.decision_reason),
            result: Set(input.result),
            decided_at: Set(input.decided_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ExecutionQueueItem>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ExecutionQueueFilter) -> BizResult<Vec<ExecutionQueueItem>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: ExecutionQueueFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateExecutionQueueItem) -> BizResult<ExecutionQueueItem> {
        let model = execution_queue::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            decision_reason: Set(input.decision_reason),
            result: Set(input.result),
            decided_at: Set(input.decided_at),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: ExecutionQueueFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<execution_queue::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(execution_queue::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(execution_queue::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(execution_queue::Column::RunId.eq(run_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(execution_queue::Column::Status.eq(status.as_str()));
        }

        if let Some(action_type) = filter.action_type {
            query = query.filter(execution_queue::Column::ActionType.eq(action_type));
        }

        query = query.order_by_desc(execution_queue::Column::CreatedAt);

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

    fn from_model(model: execution_queue::Model) -> BizResult<ExecutionQueueItem> {
        Ok(ExecutionQueueItem {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            workspace_task_id: model.workspace_task_id,
            action_type: model.action_type,
            description: model.description,
            platform: model.platform,
            payload: model.payload,
            expected_impact: model.expected_impact,
            status: ExecutionQueueStatus::try_from(model.status.as_str())?,
            decision_reason: model.decision_reason,
            result: model.result,
            created_at: model.created_at,
            decided_at: model.decided_at,
        })
    }
}
