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
    entity::tool_approval_queue,
    table::tool_approval_queue::dto::{
        CreateToolApproval, ToolApproval, ToolApprovalFilter, ToolApprovalStatus,
        UpdateToolApprovalDecision,
    },
};

db_core::impl_repository!(
    ToolApprovalQueueRepo,
    tool_approval_queue::Entity,
    tool_approval_queue::Model
);

pub struct ToolApprovalQueueService {
    repo: ToolApprovalQueueRepo,
}

impl ToolApprovalQueueService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ToolApprovalQueueRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateToolApproval) -> BizResult<ToolApproval> {
        let model = tool_approval_queue::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            conversation_id: Set(input.conversation_id),
            request_role: Set(input.request_role),
            skill_name: Set(input.skill_name),
            tool_call_id: Set(input.tool_call_id),
            risk_level: Set(input.risk_level),
            args: Set(input.args),
            status: Set(input.status.as_str().to_string()),
            reason: Set(input.reason),
            action_key: Set(input.action_key),
            result: Set(input.result),
            approved_by: Set(input.approved_by),
            approved_at: Set(input.approved_at),
            executed_at: Set(input.executed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ToolApproval>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_action_key(&self, action_key: &str) -> BizResult<Option<ToolApproval>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(tool_approval_queue::Column::ActionKey.eq(action_key)),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ToolApprovalFilter) -> BizResult<Vec<ToolApproval>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: ToolApprovalFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update_decision(
        &self,
        input: UpdateToolApprovalDecision,
    ) -> BizResult<ToolApproval> {
        let model = tool_approval_queue::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            reason: Set(input.reason),
            result: Set(input.result),
            approved_by: Set(input.approved_by),
            approved_at: Set(input.approved_at),
            executed_at: Set(input.executed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: ToolApprovalFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<tool_approval_queue::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(tool_approval_queue::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(tool_approval_queue::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(tool_approval_queue::Column::RunId.eq(run_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(tool_approval_queue::Column::Status.eq(status.as_str()));
        }

        if let Some(skill_name) = filter.skill_name {
            query = query.filter(tool_approval_queue::Column::SkillName.eq(skill_name));
        }

        if let Some(risk_level) = filter.risk_level {
            query = query.filter(tool_approval_queue::Column::RiskLevel.eq(risk_level));
        }

        query = query.order_by_desc(tool_approval_queue::Column::CreatedAt);

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

    fn from_model(model: tool_approval_queue::Model) -> BizResult<ToolApproval> {
        Ok(ToolApproval {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            conversation_id: model.conversation_id,
            request_role: model.request_role,
            skill_name: model.skill_name,
            tool_call_id: model.tool_call_id,
            risk_level: model.risk_level,
            args: model.args,
            status: ToolApprovalStatus::try_from(model.status.as_str())?,
            reason: model.reason,
            action_key: model.action_key,
            result: model.result,
            approved_by: model.approved_by,
            approved_at: model.approved_at,
            executed_at: model.executed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
