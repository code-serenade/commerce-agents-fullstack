use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::agent_quality_checks,
    table::agent_quality_checks::dto::{
        AgentQualityCheck, AgentQualityCheckFilter, CreateAgentQualityCheck,
    },
};

db_core::impl_repository!(
    AgentQualityCheckRepo,
    agent_quality_checks::Entity,
    agent_quality_checks::Model
);

pub struct AgentQualityCheckService {
    repo: AgentQualityCheckRepo,
}

impl AgentQualityCheckService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AgentQualityCheckRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAgentQualityCheck) -> BizResult<AgentQualityCheck> {
        let model = agent_quality_checks::ActiveModel {
            id: Set(input.id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            artifact_id: Set(input.artifact_id),
            user_id: Set(input.user_id),
            role: Set(input.role),
            score: Set(input.score),
            passed: Set(input.passed),
            dimensions: Set(input.dimensions),
            issues: Set(input.issues),
            suggestions: Set(input.suggestions),
            source: Set(input.source),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<AgentQualityCheck>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn list(&self, filter: AgentQualityCheckFilter) -> BizResult<Vec<AgentQualityCheck>> {
        let query = self.apply_filter(filter, true);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn count(&self, filter: AgentQualityCheckFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    fn apply_filter(
        &self,
        filter: AgentQualityCheckFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<agent_quality_checks::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(agent_quality_checks::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(agent_quality_checks::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(agent_quality_checks::Column::RunId.eq(run_id));
        }

        if let Some(artifact_id) = filter.artifact_id {
            query = query.filter(agent_quality_checks::Column::ArtifactId.eq(artifact_id));
        }

        if let Some(role) = filter.role {
            query = query.filter(agent_quality_checks::Column::Role.eq(role));
        }

        if let Some(passed) = filter.passed {
            query = query.filter(agent_quality_checks::Column::Passed.eq(passed));
        }

        query = query.order_by_desc(agent_quality_checks::Column::CreatedAt);

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

    fn from_model(model: agent_quality_checks::Model) -> AgentQualityCheck {
        AgentQualityCheck {
            id: model.id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            artifact_id: model.artifact_id,
            user_id: model.user_id,
            role: model.role,
            score: model.score,
            passed: model.passed,
            dimensions: model.dimensions,
            issues: model.issues,
            suggestions: model.suggestions,
            source: model.source,
            metadata: model.metadata,
            created_at: model.created_at,
        }
    }
}
