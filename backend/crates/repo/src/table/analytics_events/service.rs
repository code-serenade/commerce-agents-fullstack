use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, TryInsertResult,
};
use time::OffsetDateTime;

use crate::{
    entity::analytics_events,
    table::analytics_events::dto::{AnalyticsEvent, CreateAnalyticsEvent},
};

db_core::impl_repository!(
    AnalyticsEventRepo,
    analytics_events::Entity,
    analytics_events::Model
);

pub struct AnalyticsEventService {
    repo: AnalyticsEventRepo,
}

impl AnalyticsEventService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AnalyticsEventRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAnalyticsEvent) -> BizResult<AnalyticsEvent> {
        let model = Self::active_model(input);

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn create_if_event_id_absent(
        &self,
        input: CreateAnalyticsEvent,
    ) -> BizResult<Option<AnalyticsEvent>> {
        let id = input.id;
        let event_id = input.event_id.clone();
        let model = Self::active_model(input);
        let result = analytics_events::Entity::insert(model)
            .on_conflict_do_nothing_on([analytics_events::Column::EventId])
            .exec_without_returning(self.repo.db())
            .await
            .map_err(Error::from)?;

        match result {
            TryInsertResult::Inserted(rows_affected) if rows_affected > 0 => {
                match self.get_by_event_id(&event_id).await? {
                    Some(event) if event.id == id => Ok(Some(event)),
                    _ => Ok(None),
                }
            }
            TryInsertResult::Inserted(_) => Ok(None),
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    fn active_model(input: CreateAnalyticsEvent) -> analytics_events::ActiveModel {
        analytics_events::ActiveModel {
            id: Set(input.id),
            event_id: Set(input.event_id),
            event_type: Set(input.event_type),
            occurred_at: Set(input.occurred_at),
            actor_kind: Set(input.actor_kind),
            user_id: Set(input.user_id),
            anonymous_id: Set(input.anonymous_id),
            session_id: Set(input.session_id),
            school_name: Set(input.school_name),
            team_name: Set(input.team_name),
            route_path: Set(input.route_path),
            page_title: Set(input.page_title),
            referrer: Set(input.referrer),
            source_kind: Set(input.source_kind),
            source_host: Set(input.source_host),
            utm_source: Set(input.utm_source),
            utm_medium: Set(input.utm_medium),
            utm_campaign: Set(input.utm_campaign),
            utm_content: Set(input.utm_content),
            utm_term: Set(input.utm_term),
            feature_key: Set(input.feature_key),
            device_type: Set(input.device_type),
            browser_name: Set(input.browser_name),
            os_name: Set(input.os_name),
            viewport_width: Set(input.viewport_width),
            viewport_height: Set(input.viewport_height),
            status: Set(input.status),
            metadata: Set(input.metadata),
            ..Default::default()
        }
    }

    pub async fn get_by_event_id(&self, event_id: &str) -> BizResult<Option<AnalyticsEvent>> {
        let query = self
            .repo
            .query()
            .filter(analytics_events::Column::EventId.eq(event_id.to_owned()));

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
    ) -> BizResult<Vec<AnalyticsEvent>> {
        let query = self
            .repo
            .query()
            .filter(analytics_events::Column::ReceivedAt.gte(from))
            .filter(analytics_events::Column::ReceivedAt.lte(to))
            .order_by_asc(analytics_events::Column::ReceivedAt);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    fn from_model(model: analytics_events::Model) -> BizResult<AnalyticsEvent> {
        Ok(AnalyticsEvent {
            id: model.id,
            event_id: model.event_id,
            event_type: model.event_type,
            occurred_at: model.occurred_at,
            received_at: model.received_at,
            actor_kind: model.actor_kind,
            user_id: model.user_id,
            anonymous_id: model.anonymous_id,
            session_id: model.session_id,
            school_name: model.school_name,
            team_name: model.team_name,
            route_path: model.route_path,
            page_title: model.page_title,
            referrer: model.referrer,
            source_kind: model.source_kind,
            source_host: model.source_host,
            utm_source: model.utm_source,
            utm_medium: model.utm_medium,
            utm_campaign: model.utm_campaign,
            utm_content: model.utm_content,
            utm_term: model.utm_term,
            feature_key: model.feature_key,
            device_type: model.device_type,
            browser_name: model.browser_name,
            os_name: model.os_name,
            viewport_width: model.viewport_width,
            viewport_height: model.viewport_height,
            status: model.status,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
