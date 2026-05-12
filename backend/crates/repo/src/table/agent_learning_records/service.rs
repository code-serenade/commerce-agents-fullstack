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
    entity::agent_learning_records,
    table::agent_learning_records::dto::{
        AgentLearningRecord, AgentLearningRecordFilter, AgentLearningRecordStatus,
        CreateAgentLearningRecord, UpdateAgentLearningRecord,
    },
};

db_core::impl_repository!(
    AgentLearningRecordRepo,
    agent_learning_records::Entity,
    agent_learning_records::Model
);

pub struct AgentLearningRecordService {
    repo: AgentLearningRecordRepo,
}

impl AgentLearningRecordService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AgentLearningRecordRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAgentLearningRecord) -> BizResult<AgentLearningRecord> {
        let model = agent_learning_records::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            quality_check_id: Set(input.quality_check_id),
            role: Set(input.role),
            category: Set(input.category),
            content: Set(input.content),
            confidence: Set(input.confidence),
            source_action: Set(input.source_action),
            feedback_rating: Set(input.feedback_rating),
            feedback_comment: Set(input.feedback_comment),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<AgentLearningRecord>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn update(&self, input: UpdateAgentLearningRecord) -> BizResult<AgentLearningRecord> {
        let model = agent_learning_records::ActiveModel {
            id: Unchanged(input.id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            quality_check_id: Set(input.quality_check_id),
            role: Set(input.role),
            category: Set(input.category),
            content: Set(input.content),
            confidence: Set(input.confidence),
            source_action: Set(input.source_action),
            feedback_rating: Set(input.feedback_rating),
            feedback_comment: Set(input.feedback_comment),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    pub async fn list(
        &self,
        filter: AgentLearningRecordFilter,
    ) -> BizResult<Vec<AgentLearningRecord>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: AgentLearningRecordFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    fn apply_filter(
        &self,
        filter: AgentLearningRecordFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<agent_learning_records::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(agent_learning_records::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(agent_learning_records::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(agent_learning_records::Column::RunId.eq(run_id));
        }

        if let Some(quality_check_id) = filter.quality_check_id {
            query =
                query.filter(agent_learning_records::Column::QualityCheckId.eq(quality_check_id));
        }

        if let Some(role) = filter.role {
            query = query.filter(agent_learning_records::Column::Role.eq(role));
        }

        if let Some(category) = filter.category {
            query = query.filter(agent_learning_records::Column::Category.eq(category));
        }

        if let Some(source_action) = filter.source_action {
            query = query.filter(agent_learning_records::Column::SourceAction.eq(source_action));
        }

        if let Some(status) = filter.status {
            query = query.filter(agent_learning_records::Column::Status.eq(status.as_str()));
        }

        query = query.order_by_desc(agent_learning_records::Column::UpdatedAt);

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

    fn from_model(model: agent_learning_records::Model) -> BizResult<AgentLearningRecord> {
        Ok(AgentLearningRecord {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            quality_check_id: model.quality_check_id,
            role: model.role,
            category: model.category,
            content: model.content,
            confidence: model.confidence,
            source_action: model.source_action,
            feedback_rating: model.feedback_rating,
            feedback_comment: model.feedback_comment,
            status: AgentLearningRecordStatus::try_from(model.status.as_str())?,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
