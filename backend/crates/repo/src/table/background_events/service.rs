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
    entity::background_events,
    table::background_events::dto::{
        BackgroundEvent, BackgroundEventFilter, CreateBackgroundEvent,
        UpdateBackgroundEventConsumed,
    },
};

db_core::impl_repository!(
    BackgroundEventRepo,
    background_events::Entity,
    background_events::Model
);

pub struct BackgroundEventService {
    repo: BackgroundEventRepo,
}

impl BackgroundEventService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: BackgroundEventRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateBackgroundEvent) -> BizResult<BackgroundEvent> {
        let model = background_events::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            conversation_id: Set(input.conversation_id),
            event_type: Set(input.event_type),
            title: Set(input.title),
            data: Set(input.data),
            consumed: Set(input.consumed),
            consumed_at: Set(input.consumed_at),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<BackgroundEvent>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn list(&self, filter: BackgroundEventFilter) -> BizResult<Vec<BackgroundEvent>> {
        let query = self.apply_filter(filter, true);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn count(&self, filter: BackgroundEventFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update_consumed(
        &self,
        input: UpdateBackgroundEventConsumed,
    ) -> BizResult<BackgroundEvent> {
        let model = background_events::ActiveModel {
            id: Unchanged(input.id),
            consumed: Set(input.consumed),
            consumed_at: Set(input.consumed_at),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    fn apply_filter(
        &self,
        filter: BackgroundEventFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<background_events::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(background_events::Column::UserId.eq(filter.user_id));

        if let Some(conversation_id) = filter.conversation_id {
            query = query.filter(background_events::Column::ConversationId.eq(conversation_id));
        }

        if let Some(event_type) = filter.event_type {
            query = query.filter(background_events::Column::EventType.eq(event_type));
        }

        if let Some(consumed) = filter.consumed {
            query = query.filter(background_events::Column::Consumed.eq(consumed));
        }

        query = query.order_by_desc(background_events::Column::CreatedAt);

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

    fn from_model(model: background_events::Model) -> BackgroundEvent {
        BackgroundEvent {
            id: model.id,
            user_id: model.user_id,
            conversation_id: model.conversation_id,
            event_type: model.event_type,
            title: model.title,
            data: model.data,
            consumed: model.consumed,
            consumed_at: model.consumed_at,
            created_at: model.created_at,
        }
    }
}
