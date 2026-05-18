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
    entity::agent_trust_scores,
    table::agent_trust_scores::dto::{
        AgentTrustLevel, AgentTrustScore, AgentTrustScoreFilter, CreateAgentTrustScore,
        UpdateAgentTrustScore,
    },
};

db_core::impl_repository!(
    AgentTrustScoreRepo,
    agent_trust_scores::Entity,
    agent_trust_scores::Model
);

pub struct AgentTrustScoreService {
    repo: AgentTrustScoreRepo,
}

impl AgentTrustScoreService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AgentTrustScoreRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAgentTrustScore) -> BizResult<AgentTrustScore> {
        let model = agent_trust_scores::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            role: Set(input.role),
            skill_id: Set(input.skill_id),
            score: Set(input.score),
            trust_level: Set(input.trust_level.as_str().to_string()),
            previous_score: Set(input.previous_score),
            previous_level: Set(input.previous_level),
            delta: Set(input.delta),
            reason: Set(input.reason),
            source: Set(input.source),
            quality_check_id: Set(input.quality_check_id),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<AgentTrustScore>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn find_by_role(
        &self,
        user_id: Uuid,
        role: &str,
        skill_id: Option<&str>,
    ) -> BizResult<Option<AgentTrustScore>> {
        let mut query = self
            .repo
            .query()
            .filter(agent_trust_scores::Column::UserId.eq(user_id))
            .filter(agent_trust_scores::Column::Role.eq(role.to_string()));

        query = match skill_id {
            Some(skill_id) => query.filter(agent_trust_scores::Column::SkillId.eq(skill_id)),
            None => query.filter(agent_trust_scores::Column::SkillId.is_null()),
        };

        self.repo
            .select_one(query)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: AgentTrustScoreFilter) -> BizResult<Vec<AgentTrustScore>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: AgentTrustScoreFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateAgentTrustScore) -> BizResult<AgentTrustScore> {
        let model = agent_trust_scores::ActiveModel {
            id: Unchanged(input.id),
            score: Set(input.score),
            trust_level: Set(input.trust_level.as_str().to_string()),
            previous_score: Set(input.previous_score),
            previous_level: Set(input.previous_level),
            delta: Set(input.delta),
            reason: Set(input.reason),
            source: Set(input.source),
            quality_check_id: Set(input.quality_check_id),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: AgentTrustScoreFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<agent_trust_scores::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(agent_trust_scores::Column::UserId.eq(filter.user_id));

        if let Some(role) = filter.role {
            query = query.filter(agent_trust_scores::Column::Role.eq(role));
        }

        if let Some(skill_id) = filter.skill_id {
            query = query.filter(agent_trust_scores::Column::SkillId.eq(skill_id));
        }

        query = query.order_by_desc(agent_trust_scores::Column::UpdatedAt);

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

    fn from_model(model: agent_trust_scores::Model) -> BizResult<AgentTrustScore> {
        Ok(AgentTrustScore {
            id: model.id,
            user_id: model.user_id,
            role: model.role,
            skill_id: model.skill_id,
            score: model.score,
            trust_level: AgentTrustLevel::try_from(model.trust_level.as_str())?,
            previous_score: model.previous_score,
            previous_level: model.previous_level,
            delta: model.delta,
            reason: model.reason,
            source: model.source,
            quality_check_id: model.quality_check_id,
            metadata: model.metadata,
            updated_at: model.updated_at,
        })
    }
}
