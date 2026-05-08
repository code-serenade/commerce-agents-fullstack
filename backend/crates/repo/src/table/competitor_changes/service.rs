use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{
    entity::competitor_changes,
    table::competitor_changes::dto::{
        CompetitorChange, CompetitorChangeFilter, CreateCompetitorChange,
    },
};

db_core::impl_repository!(
    CompetitorChangeRepo,
    competitor_changes::Entity,
    competitor_changes::Model
);

pub struct CompetitorChangeService {
    repo: CompetitorChangeRepo,
}

impl CompetitorChangeService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: CompetitorChangeRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateCompetitorChange) -> BizResult<CompetitorChange> {
        let model = competitor_changes::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            competitor_id: Set(input.competitor_id),
            change_type: Set(input.change_type),
            old_value: Set(input.old_value),
            new_value: Set(input.new_value),
            summary: Set(input.summary),
            source_type: Set(input.source_type),
            metadata: Set(input.metadata),
            detected_at: input.detected_at.map(Set).unwrap_or_default(),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list(&self, filter: CompetitorChangeFilter) -> BizResult<Vec<CompetitorChange>> {
        let mut query = self
            .repo
            .query()
            .filter(competitor_changes::Column::UserId.eq(filter.user_id))
            .filter(competitor_changes::Column::ProductId.eq(filter.product_id));

        if let Some(competitor_id) = filter.competitor_id {
            query = query.filter(competitor_changes::Column::CompetitorId.eq(competitor_id));
        }

        if let Some(change_type) = filter.change_type {
            query = query.filter(competitor_changes::Column::ChangeType.eq(change_type));
        }

        query = query.order_by_desc(competitor_changes::Column::DetectedAt);

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    fn from_model(model: competitor_changes::Model) -> BizResult<CompetitorChange> {
        Ok(CompetitorChange {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            competitor_id: model.competitor_id,
            change_type: model.change_type,
            old_value: model.old_value,
            new_value: model.new_value,
            summary: model.summary,
            source_type: model.source_type,
            metadata: model.metadata,
            detected_at: model.detected_at,
            created_at: model.created_at,
        })
    }
}
