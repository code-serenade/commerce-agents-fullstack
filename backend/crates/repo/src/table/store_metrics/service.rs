use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::store_metrics,
    table::store_metrics::dto::{
        StoreMetric, StoreMetricAggregate, StoreMetricFilter, UpsertStoreMetric,
    },
};

db_core::impl_repository!(StoreMetricRepo, store_metrics::Entity, store_metrics::Model);

pub struct StoreMetricService {
    repo: StoreMetricRepo,
}

impl StoreMetricService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StoreMetricRepo::new(db),
        }
    }

    pub async fn upsert(&self, input: UpsertStoreMetric) -> BizResult<StoreMetric> {
        if let Some(existing) = self
            .get_dedup(
                input.user_id,
                input.platform.as_str(),
                input.metric_date.as_str(),
                input.metric_key.as_str(),
                input.dimension_key.as_str(),
                input.dimension_value.as_str(),
            )
            .await?
        {
            let model = store_metrics::ActiveModel {
                id: Unchanged(existing.id),
                metric_value: Set(input.metric_value),
                source: Set(input.source),
                ..Default::default()
            };
            return Self::from_model(self.repo.update(model).await?);
        }

        let model = store_metrics::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            platform: Set(input.platform),
            metric_date: Set(input.metric_date),
            metric_key: Set(input.metric_key),
            metric_value: Set(input.metric_value),
            dimension_key: Set(input.dimension_key),
            dimension_value: Set(input.dimension_value),
            source: Set(input.source),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<StoreMetric>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: StoreMetricFilter) -> BizResult<Vec<StoreMetric>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: StoreMetricFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn aggregate_since(
        &self,
        user_id: Uuid,
        since_date: &str,
    ) -> BizResult<Vec<StoreMetricAggregate>> {
        let rows = store_metrics::Entity::find()
            .select_only()
            .column(store_metrics::Column::Platform)
            .column(store_metrics::Column::MetricKey)
            .column_as(store_metrics::Column::MetricValue.sum(), "total")
            .filter(store_metrics::Column::UserId.eq(user_id))
            .filter(store_metrics::Column::MetricDate.gte(since_date))
            .group_by(store_metrics::Column::Platform)
            .group_by(store_metrics::Column::MetricKey)
            .into_tuple::<(String, String, f64)>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|(platform, metric_key, total)| StoreMetricAggregate {
                platform,
                metric_key,
                total,
            })
            .collect())
    }

    pub async fn get_latest_metric_date(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> BizResult<Option<String>> {
        let item = store_metrics::Entity::find()
            .filter(store_metrics::Column::UserId.eq(user_id))
            .filter(store_metrics::Column::Platform.eq(platform))
            .order_by_desc(store_metrics::Column::MetricDate)
            .one(self.repo.db())
            .await
            .map_err(Error::from)?;
        Ok(item.map(|item| item.metric_date))
    }

    async fn get_dedup(
        &self,
        user_id: Uuid,
        platform: &str,
        metric_date: &str,
        metric_key: &str,
        dimension_key: &str,
        dimension_value: &str,
    ) -> BizResult<Option<StoreMetric>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(store_metrics::Column::UserId.eq(user_id))
                    .filter(store_metrics::Column::Platform.eq(platform))
                    .filter(store_metrics::Column::MetricDate.eq(metric_date))
                    .filter(store_metrics::Column::MetricKey.eq(metric_key))
                    .filter(store_metrics::Column::DimensionKey.eq(dimension_key))
                    .filter(store_metrics::Column::DimensionValue.eq(dimension_value)),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    fn apply_filter(
        &self,
        filter: StoreMetricFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<store_metrics::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(store_metrics::Column::UserId.eq(filter.user_id));

        if let Some(platform) = filter.platform {
            query = query.filter(store_metrics::Column::Platform.eq(platform));
        }

        if let Some(metric_key) = filter.metric_key {
            query = query.filter(store_metrics::Column::MetricKey.eq(metric_key));
        }

        if let Some(metric_date_from) = filter.metric_date_from {
            query = query.filter(store_metrics::Column::MetricDate.gte(metric_date_from));
        }

        if let Some(metric_date_to) = filter.metric_date_to {
            query = query.filter(store_metrics::Column::MetricDate.lte(metric_date_to));
        }

        if let Some(source) = filter.source {
            query = query.filter(store_metrics::Column::Source.eq(source));
        }

        query = query
            .order_by_desc(store_metrics::Column::MetricDate)
            .order_by_asc(store_metrics::Column::Platform);

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

    fn from_model(model: store_metrics::Model) -> BizResult<StoreMetric> {
        Ok(StoreMetric {
            id: model.id,
            user_id: model.user_id,
            platform: model.platform,
            metric_date: model.metric_date,
            metric_key: model.metric_key,
            metric_value: model.metric_value,
            dimension_key: model.dimension_key,
            dimension_value: model.dimension_value,
            source: model.source,
            created_at: model.created_at,
        })
    }
}
