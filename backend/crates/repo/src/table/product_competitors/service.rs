use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::product_competitors,
    table::product_competitors::dto::{
        CreateProductCompetitor, ProductCompetitor, ProductCompetitorFilter,
        ProductCompetitorStatus, UpdateProductCompetitor,
    },
};

db_core::impl_repository!(
    ProductCompetitorRepo,
    product_competitors::Entity,
    product_competitors::Model
);

pub struct ProductCompetitorService {
    repo: ProductCompetitorRepo,
}

impl ProductCompetitorService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ProductCompetitorRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateProductCompetitor) -> BizResult<ProductCompetitor> {
        let model = product_competitors::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            name: Set(input.name),
            platform: Set(input.platform),
            url: Set(input.url),
            price: Set(input.price),
            rating: Set(input.rating),
            monthly_sales: Set(input.monthly_sales),
            notes: Set(input.notes),
            search_results: Set(input.search_results),
            status: Set(ProductCompetitorStatus::Active.as_str().to_string()),
            metadata: Set(input.metadata),
            last_searched_at: Set(input.last_searched_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ProductCompetitor>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ProductCompetitorFilter) -> BizResult<Vec<ProductCompetitor>> {
        let mut query = self
            .repo
            .query()
            .filter(product_competitors::Column::UserId.eq(filter.user_id))
            .filter(product_competitors::Column::ProductId.eq(filter.product_id));

        if let Some(status) = filter.status {
            query = query.filter(product_competitors::Column::Status.eq(status.as_str()));
        }

        if let Some(platform) = filter.platform {
            query = query.filter(product_competitors::Column::Platform.eq(platform));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(product_competitors::Column::Name.contains(keyword.clone()))
                .add(product_competitors::Column::Platform.contains(keyword.clone()))
                .add(product_competitors::Column::Url.contains(keyword.clone()))
                .add(product_competitors::Column::MonthlySales.contains(keyword.clone()))
                .add(product_competitors::Column::Notes.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(product_competitors::Column::UpdatedAt);

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

    pub async fn update(&self, input: UpdateProductCompetitor) -> BizResult<ProductCompetitor> {
        let model = product_competitors::ActiveModel {
            id: Unchanged(input.id),
            name: Set(input.name),
            platform: Set(input.platform),
            url: Set(input.url),
            price: Set(input.price),
            rating: Set(input.rating),
            monthly_sales: Set(input.monthly_sales),
            notes: Set(input.notes),
            search_results: Set(input.search_results),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            last_searched_at: Set(input.last_searched_at),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: product_competitors::Model) -> BizResult<ProductCompetitor> {
        Ok(ProductCompetitor {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            name: model.name,
            platform: model.platform,
            url: model.url,
            price: model.price,
            rating: model.rating,
            monthly_sales: model.monthly_sales,
            notes: model.notes,
            search_results: model.search_results,
            status: ProductCompetitorStatus::try_from(model.status)?,
            metadata: model.metadata,
            last_searched_at: model.last_searched_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
