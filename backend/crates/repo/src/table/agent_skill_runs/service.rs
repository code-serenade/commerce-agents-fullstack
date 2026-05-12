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
    entity::agent_skill_runs,
    table::agent_skill_runs::dto::{
        AgentSkillRun, AgentSkillRunFilter, AgentSkillRunStatus, CreateAgentSkillRun,
        UpdateAgentSkillRun,
    },
};

db_core::impl_repository!(
    AgentSkillRunRepo,
    agent_skill_runs::Entity,
    agent_skill_runs::Model
);

pub struct AgentSkillRunService {
    repo: AgentSkillRunRepo,
}

impl AgentSkillRunService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AgentSkillRunRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAgentSkillRun) -> BizResult<AgentSkillRun> {
        let model = agent_skill_runs::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            session_id: Set(input.session_id),
            worker_id: Set(input.worker_id),
            task_id: Set(input.task_id),
            request_id: Set(input.request_id),
            skill_id: Set(input.skill_id),
            role: Set(input.role),
            status: Set(input.status.as_str().to_string()),
            reason: Set(input.reason),
            summary: Set(input.summary),
            error: Set(input.error),
            input: Set(input.input),
            output: Set(input.output),
            execution_mode: Set(input.execution_mode),
            risk_level: Set(input.risk_level),
            approval_required: Set(input.approval_required),
            metadata: Set(input.metadata),
            requested_at: Set(input.requested_at),
            completed_at: Set(input.completed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_request(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Uuid,
        request_id: &str,
    ) -> BizResult<Option<AgentSkillRun>> {
        self.repo
            .query()
            .filter(agent_skill_runs::Column::UserId.eq(user_id))
            .filter(agent_skill_runs::Column::WorkspaceId.eq(workspace_id))
            .filter(agent_skill_runs::Column::RunId.eq(run_id))
            .filter(agent_skill_runs::Column::RequestId.eq(request_id))
            .one(self.repo.db())
            .await
            .map_err(Error::from)?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: AgentSkillRunFilter) -> BizResult<Vec<AgentSkillRun>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: AgentSkillRunFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateAgentSkillRun) -> BizResult<AgentSkillRun> {
        let model = agent_skill_runs::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            reason: Set(input.reason),
            summary: Set(input.summary),
            error: Set(input.error),
            input: Set(input.input),
            output: Set(input.output),
            execution_mode: Set(input.execution_mode),
            risk_level: Set(input.risk_level),
            approval_required: Set(input.approval_required),
            metadata: Set(input.metadata),
            completed_at: Set(input.completed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: AgentSkillRunFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<agent_skill_runs::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(agent_skill_runs::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(agent_skill_runs::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(agent_skill_runs::Column::RunId.eq(run_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(agent_skill_runs::Column::Status.eq(status.as_str()));
        }

        if let Some(skill_id) = filter.skill_id {
            query = query.filter(agent_skill_runs::Column::SkillId.eq(skill_id));
        }

        if let Some(role) = filter.role {
            query = query.filter(agent_skill_runs::Column::Role.eq(role));
        }

        query = query.order_by_desc(agent_skill_runs::Column::RequestedAt);

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

    fn from_model(model: agent_skill_runs::Model) -> BizResult<AgentSkillRun> {
        Ok(AgentSkillRun {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            session_id: model.session_id,
            worker_id: model.worker_id,
            task_id: model.task_id,
            request_id: model.request_id,
            skill_id: model.skill_id,
            role: model.role,
            status: AgentSkillRunStatus::try_from(model.status.as_str())?,
            reason: model.reason,
            summary: model.summary,
            error: model.error,
            input: model.input,
            output: model.output,
            execution_mode: model.execution_mode,
            risk_level: model.risk_level,
            approval_required: model.approval_required,
            metadata: model.metadata,
            requested_at: model.requested_at,
            completed_at: model.completed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
