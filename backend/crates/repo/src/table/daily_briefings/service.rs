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
    entity::daily_briefings,
    table::daily_briefings::dto::{
        DailyBriefing, DailyBriefingFilter, UpdateDailyBriefing, UpsertDailyBriefing,
    },
};

db_core::impl_repository!(
    DailyBriefingRepo,
    daily_briefings::Entity,
    daily_briefings::Model
);

pub struct DailyBriefingService {
    repo: DailyBriefingRepo,
}

impl DailyBriefingService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: DailyBriefingRepo::new(db),
        }
    }

    pub async fn upsert(&self, input: UpsertDailyBriefing) -> BizResult<DailyBriefing> {
        if let Some(existing) = self
            .get_by_user_and_date(input.user_id, input.briefing_date.as_str())
            .await?
        {
            return self
                .update(UpdateDailyBriefing {
                    id: existing.id,
                    title: input.title,
                    summary: input.summary,
                    sections: input.sections,
                    product_summary: input.product_summary,
                    alert_summary: input.alert_summary,
                    task_summary: input.task_summary,
                    source: input.source,
                    status: input.status,
                    archived: input.archived,
                })
                .await;
        }

        let model = daily_briefings::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            briefing_date: Set(input.briefing_date),
            title: Set(input.title),
            summary: Set(input.summary),
            sections: Set(input.sections),
            product_summary: Set(input.product_summary),
            alert_summary: Set(input.alert_summary),
            task_summary: Set(input.task_summary),
            source: Set(input.source),
            status: Set(input.status),
            archived: Set(input.archived),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<DailyBriefing>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn get_by_user_and_date(
        &self,
        user_id: Uuid,
        briefing_date: &str,
    ) -> BizResult<Option<DailyBriefing>> {
        Ok(self
            .repo
            .select_one(
                self.repo
                    .query()
                    .filter(daily_briefings::Column::UserId.eq(user_id))
                    .filter(daily_briefings::Column::BriefingDate.eq(briefing_date)),
            )
            .await?
            .map(Self::from_model))
    }

    pub async fn list(&self, filter: DailyBriefingFilter) -> BizResult<Vec<DailyBriefing>> {
        let query = self.apply_filter(filter, true);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn count(&self, filter: DailyBriefingFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateDailyBriefing) -> BizResult<DailyBriefing> {
        let model = daily_briefings::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            summary: Set(input.summary),
            sections: Set(input.sections),
            product_summary: Set(input.product_summary),
            alert_summary: Set(input.alert_summary),
            task_summary: Set(input.task_summary),
            source: Set(input.source),
            status: Set(input.status),
            archived: Set(input.archived),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    fn apply_filter(
        &self,
        filter: DailyBriefingFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<daily_briefings::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(daily_briefings::Column::UserId.eq(filter.user_id));

        if let Some(archived) = filter.archived {
            query = query.filter(daily_briefings::Column::Archived.eq(archived));
        }

        if let Some(status) = filter.status {
            query = query.filter(daily_briefings::Column::Status.eq(status));
        }

        if let Some(date_from) = filter.date_from {
            query = query.filter(daily_briefings::Column::BriefingDate.gte(date_from));
        }

        if let Some(date_to) = filter.date_to {
            query = query.filter(daily_briefings::Column::BriefingDate.lte(date_to));
        }

        query = query.order_by_desc(daily_briefings::Column::BriefingDate);

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

    fn from_model(model: daily_briefings::Model) -> DailyBriefing {
        DailyBriefing {
            id: model.id,
            user_id: model.user_id,
            briefing_date: model.briefing_date,
            title: model.title,
            summary: model.summary,
            sections: model.sections,
            product_summary: model.product_summary,
            alert_summary: model.alert_summary,
            task_summary: model.task_summary,
            source: model.source,
            status: model.status,
            archived: model.archived,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
