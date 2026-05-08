use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::audit_logs,
    table::audit_logs::dto::{AuditLog, AuditLogFilter, CreateAuditLog},
};

db_core::impl_repository!(AuditLogRepo, audit_logs::Entity, audit_logs::Model);

pub struct AuditLogService {
    repo: AuditLogRepo,
}

impl AuditLogService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AuditLogRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAuditLog) -> BizResult<AuditLog> {
        let model = audit_logs::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            actor_user_id: Set(input.actor_user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            execution_queue_id: Set(input.execution_queue_id),
            tool_approval_id: Set(input.tool_approval_id),
            action: Set(input.action),
            target_type: Set(input.target_type),
            target_id: Set(input.target_id),
            status: Set(input.status),
            reason: Set(input.reason),
            request_id: Set(input.request_id),
            trace_id: Set(input.trace_id),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<AuditLog>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn list(&self, filter: AuditLogFilter) -> BizResult<Vec<AuditLog>> {
        let query = self.apply_filter(filter, true);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn count(&self, filter: AuditLogFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    fn apply_filter(
        &self,
        filter: AuditLogFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<audit_logs::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(audit_logs::Column::UserId.eq(filter.user_id));

        if let Some(actor_user_id) = filter.actor_user_id {
            query = query.filter(audit_logs::Column::ActorUserId.eq(actor_user_id));
        }

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(audit_logs::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(audit_logs::Column::RunId.eq(run_id));
        }

        if let Some(execution_queue_id) = filter.execution_queue_id {
            query = query.filter(audit_logs::Column::ExecutionQueueId.eq(execution_queue_id));
        }

        if let Some(tool_approval_id) = filter.tool_approval_id {
            query = query.filter(audit_logs::Column::ToolApprovalId.eq(tool_approval_id));
        }

        if let Some(action) = filter.action {
            query = query.filter(audit_logs::Column::Action.eq(action));
        }

        if let Some(target_type) = filter.target_type {
            query = query.filter(audit_logs::Column::TargetType.eq(target_type));
        }

        if let Some(status) = filter.status {
            query = query.filter(audit_logs::Column::Status.eq(status));
        }

        query = query.order_by_desc(audit_logs::Column::CreatedAt);

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

    fn from_model(model: audit_logs::Model) -> AuditLog {
        AuditLog {
            id: model.id,
            user_id: model.user_id,
            actor_user_id: model.actor_user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            execution_queue_id: model.execution_queue_id,
            tool_approval_id: model.tool_approval_id,
            action: model.action,
            target_type: model.target_type,
            target_id: model.target_id,
            status: model.status,
            reason: model.reason,
            request_id: model.request_id,
            trace_id: model.trace_id,
            metadata: model.metadata,
            created_at: model.created_at,
        }
    }
}
