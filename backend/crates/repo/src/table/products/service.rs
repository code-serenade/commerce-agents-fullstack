use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::products,
    table::products::dto::{CreateProduct, Product, ProductFilter, ProductStatus, UpdateProduct},
};

db_core::impl_repository!(ProductRepo, products::Entity, products::Model);

pub struct ProductService {
    repo: ProductRepo,
}

impl ProductService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ProductRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateProduct) -> BizResult<Product> {
        let model = products::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            name: Set(input.name),
            platform: Set(input.platform),
            external_id: Set(input.external_id),
            category: Set(input.category),
            brand: Set(input.brand),
            status: Set(ProductStatus::Active.as_str().to_string()),
            description: Set(input.description),
            price: Set(input.price),
            currency: Set(input.currency),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<Product>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ProductFilter) -> BizResult<Vec<Product>> {
        let mut query = self
            .repo
            .query()
            .filter(products::Column::UserId.eq(filter.user_id));

        if let Some(status) = filter.status {
            query = query.filter(products::Column::Status.eq(status.as_str()));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(products::Column::Name.contains(keyword.clone()))
                .add(products::Column::Platform.contains(keyword.clone()))
                .add(products::Column::ExternalId.contains(keyword.clone()))
                .add(products::Column::Category.contains(keyword.clone()))
                .add(products::Column::Brand.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(products::Column::UpdatedAt);

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

    pub async fn count_by_user_and_status(
        &self,
        user_id: Uuid,
        status: ProductStatus,
    ) -> BizResult<u64> {
        Ok(self
            .repo
            .query()
            .filter(products::Column::UserId.eq(user_id))
            .filter(products::Column::Status.eq(status.as_str()))
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateProduct) -> BizResult<Product> {
        let model = products::ActiveModel {
            id: Unchanged(input.id),
            name: Set(input.name),
            platform: Set(input.platform),
            external_id: Set(input.external_id),
            category: Set(input.category),
            brand: Set(input.brand),
            status: Set(input.status.as_str().to_string()),
            description: Set(input.description),
            price: Set(input.price),
            currency: Set(input.currency),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: products::Model) -> BizResult<Product> {
        Ok(Product {
            id: model.id,
            user_id: model.user_id,
            name: model.name,
            platform: model.platform,
            external_id: model.external_id,
            category: model.category,
            brand: model.brand,
            status: ProductStatus::try_from(model.status)?,
            description: model.description,
            price: model.price,
            currency: model.currency,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
