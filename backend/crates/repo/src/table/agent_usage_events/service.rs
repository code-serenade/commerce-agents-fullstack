use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, FromQueryResult, QueryFilter, QuerySelect};

use crate::{
    entity::agent_usage_events,
    table::agent_usage_events::dto::{
        AgentUsageEvent, CreateAgentUsageEvent, TeamUsageEventStats, UserUsageEventStats,
    },
};

db_core::impl_repository!(
    AgentUsageEventRepo,
    agent_usage_events::Entity,
    agent_usage_events::Model
);

pub struct AgentUsageEventService {
    repo: AgentUsageEventRepo,
}

#[derive(Debug, FromQueryResult)]
struct TeamUsageEventStatsRow {
    school_name: String,
    team_name: String,
    question_used: Option<i64>,
    token_used: Option<i64>,
    last_used_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, FromQueryResult)]
struct UserUsageEventStatsRow {
    user_id: uuid::Uuid,
    question_used: Option<i64>,
    token_used: Option<i64>,
    last_used_at: Option<time::OffsetDateTime>,
}

impl AgentUsageEventService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AgentUsageEventRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAgentUsageEvent) -> BizResult<AgentUsageEvent> {
        let model = agent_usage_events::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            school_name: Set(input.school_name),
            team_name: Set(input.team_name),
            conversation_id: Set(input.conversation_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            session_id: Set(input.session_id),
            agent_id: Set(input.agent_id),
            question_count: Set(input.question_count),
            prompt_tokens: Set(input.prompt_tokens),
            completion_tokens: Set(input.completion_tokens),
            total_tokens: Set(input.total_tokens),
            status: Set(input.status),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn team_stats(&self) -> BizResult<Vec<TeamUsageEventStats>> {
        let rows = self
            .repo
            .query()
            .select_only()
            .column(agent_usage_events::Column::SchoolName)
            .column(agent_usage_events::Column::TeamName)
            .column_as(
                agent_usage_events::Column::QuestionCount.sum(),
                "question_used",
            )
            .column_as(agent_usage_events::Column::TotalTokens.sum(), "token_used")
            .column_as(agent_usage_events::Column::CreatedAt.max(), "last_used_at")
            .group_by(agent_usage_events::Column::SchoolName)
            .group_by(agent_usage_events::Column::TeamName)
            .into_model::<TeamUsageEventStatsRow>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|row| TeamUsageEventStats {
                school_name: row.school_name,
                team_name: row.team_name,
                question_used: row.question_used.unwrap_or(0),
                token_used: row.token_used.unwrap_or(0),
                last_used_at: row.last_used_at,
            })
            .collect())
    }

    pub async fn team_stats_by_team(
        &self,
        school_name: &str,
        team_name: &str,
    ) -> BizResult<TeamUsageEventStats> {
        let mut stats = self
            .team_stats_filtered(Some(school_name), Some(team_name))
            .await?;
        Ok(stats.pop().unwrap_or_else(|| TeamUsageEventStats {
            school_name: school_name.to_owned(),
            team_name: team_name.to_owned(),
            question_used: 0,
            token_used: 0,
            last_used_at: None,
        }))
    }

    pub async fn user_stats_by_team(
        &self,
        school_name: &str,
        team_name: &str,
    ) -> BizResult<Vec<UserUsageEventStats>> {
        let rows = self
            .repo
            .query()
            .select_only()
            .column(agent_usage_events::Column::UserId)
            .column_as(
                agent_usage_events::Column::QuestionCount.sum(),
                "question_used",
            )
            .column_as(agent_usage_events::Column::TotalTokens.sum(), "token_used")
            .column_as(agent_usage_events::Column::CreatedAt.max(), "last_used_at")
            .filter(agent_usage_events::Column::SchoolName.eq(school_name.to_owned()))
            .filter(agent_usage_events::Column::TeamName.eq(team_name.to_owned()))
            .group_by(agent_usage_events::Column::UserId)
            .into_model::<UserUsageEventStatsRow>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|row| UserUsageEventStats {
                user_id: row.user_id,
                question_used: row.question_used.unwrap_or(0),
                token_used: row.token_used.unwrap_or(0),
                last_used_at: row.last_used_at,
            })
            .collect())
    }

    async fn team_stats_filtered(
        &self,
        school_name: Option<&str>,
        team_name: Option<&str>,
    ) -> BizResult<Vec<TeamUsageEventStats>> {
        let mut query = self
            .repo
            .query()
            .select_only()
            .column(agent_usage_events::Column::SchoolName)
            .column(agent_usage_events::Column::TeamName)
            .column_as(
                agent_usage_events::Column::QuestionCount.sum(),
                "question_used",
            )
            .column_as(agent_usage_events::Column::TotalTokens.sum(), "token_used")
            .column_as(agent_usage_events::Column::CreatedAt.max(), "last_used_at");

        if let Some(school_name) = school_name {
            query = query.filter(agent_usage_events::Column::SchoolName.eq(school_name.to_owned()));
        }
        if let Some(team_name) = team_name {
            query = query.filter(agent_usage_events::Column::TeamName.eq(team_name.to_owned()));
        }

        let rows = query
            .group_by(agent_usage_events::Column::SchoolName)
            .group_by(agent_usage_events::Column::TeamName)
            .into_model::<TeamUsageEventStatsRow>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|row| TeamUsageEventStats {
                school_name: row.school_name,
                team_name: row.team_name,
                question_used: row.question_used.unwrap_or(0),
                token_used: row.token_used.unwrap_or(0),
                last_used_at: row.last_used_at,
            })
            .collect())
    }

    fn from_model(model: agent_usage_events::Model) -> AgentUsageEvent {
        AgentUsageEvent {
            id: model.id,
            user_id: model.user_id,
            school_name: model.school_name,
            team_name: model.team_name,
            conversation_id: model.conversation_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            session_id: model.session_id,
            agent_id: model.agent_id,
            question_count: model.question_count,
            prompt_tokens: model.prompt_tokens,
            completion_tokens: model.completion_tokens,
            total_tokens: model.total_tokens,
            status: model.status,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
