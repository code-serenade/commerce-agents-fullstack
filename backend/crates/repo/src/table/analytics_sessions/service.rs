use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, EntityTrait, QueryFilter, QueryOrder, TryInsertResult,
};
use time::OffsetDateTime;

use crate::{
    entity::analytics_sessions,
    table::analytics_sessions::dto::{
        AnalyticsSession, CreateAnalyticsSession, UpdateAnalyticsSession,
    },
};

db_core::impl_repository!(
    AnalyticsSessionRepo,
    analytics_sessions::Entity,
    analytics_sessions::Model
);

pub struct AnalyticsSessionService {
    repo: AnalyticsSessionRepo,
}

impl AnalyticsSessionService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AnalyticsSessionRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAnalyticsSession) -> BizResult<AnalyticsSession> {
        let model = Self::create_active_model(input);

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn create_if_session_id_absent(
        &self,
        input: CreateAnalyticsSession,
    ) -> BizResult<Option<AnalyticsSession>> {
        let id = input.id;
        let session_id = input.session_id.clone();
        let model = Self::create_active_model(input);
        let result = analytics_sessions::Entity::insert(model)
            .on_conflict_do_nothing_on([analytics_sessions::Column::SessionId])
            .exec_without_returning(self.repo.db())
            .await
            .map_err(Error::from)?;

        match result {
            TryInsertResult::Inserted(rows_affected) if rows_affected > 0 => {
                match self.get_by_session_id(&session_id).await? {
                    Some(session) if session.id == id => Ok(Some(session)),
                    _ => Ok(None),
                }
            }
            TryInsertResult::Inserted(_) => Ok(None),
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    fn create_active_model(input: CreateAnalyticsSession) -> analytics_sessions::ActiveModel {
        analytics_sessions::ActiveModel {
            id: Set(input.id),
            session_id: Set(input.session_id),
            actor_kind: Set(input.actor_kind),
            user_id: Set(input.user_id),
            anonymous_id: Set(input.anonymous_id),
            school_name: Set(input.school_name),
            team_name: Set(input.team_name),
            first_seen_at: Set(input.first_seen_at),
            last_seen_at: Set(input.last_seen_at),
            entry_path: Set(input.entry_path),
            exit_path: Set(input.exit_path),
            source_kind: Set(input.source_kind),
            source_host: Set(input.source_host),
            page_view_count: Set(input.page_view_count),
            event_count: Set(input.event_count),
            ..Default::default()
        }
    }

    pub async fn update(&self, input: UpdateAnalyticsSession) -> BizResult<AnalyticsSession> {
        let model = analytics_sessions::ActiveModel {
            id: Unchanged(input.id),
            actor_kind: Set(input.actor_kind),
            user_id: Set(input.user_id),
            anonymous_id: Set(input.anonymous_id),
            school_name: Set(input.school_name),
            team_name: Set(input.team_name),
            last_seen_at: Set(input.last_seen_at),
            exit_path: Set(input.exit_path),
            source_kind: Set(input.source_kind),
            source_host: Set(input.source_host),
            page_view_count: Set(input.page_view_count),
            event_count: Set(input.event_count),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    pub async fn get_by_session_id(&self, session_id: &str) -> BizResult<Option<AnalyticsSession>> {
        let query = self
            .repo
            .query()
            .filter(analytics_sessions::Column::SessionId.eq(session_id.to_owned()));

        self.repo
            .select_one(query)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list_between(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> BizResult<Vec<AnalyticsSession>> {
        let query = self
            .repo
            .query()
            .filter(analytics_sessions::Column::FirstSeenAt.lte(to))
            .filter(analytics_sessions::Column::LastSeenAt.gte(from))
            .order_by_asc(analytics_sessions::Column::FirstSeenAt);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    fn from_model(model: analytics_sessions::Model) -> BizResult<AnalyticsSession> {
        Ok(AnalyticsSession {
            id: model.id,
            session_id: model.session_id,
            actor_kind: model.actor_kind,
            user_id: model.user_id,
            anonymous_id: model.anonymous_id,
            school_name: model.school_name,
            team_name: model.team_name,
            first_seen_at: model.first_seen_at,
            last_seen_at: model.last_seen_at,
            entry_path: model.entry_path,
            exit_path: model.exit_path,
            source_kind: model.source_kind,
            source_host: model.source_host,
            page_view_count: model.page_view_count,
            event_count: model.event_count,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
