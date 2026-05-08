use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::product_materials,
    table::product_materials::dto::{
        CreateProductMaterial, ProductMaterial, ProductMaterialFilter, ProductMaterialStatus,
        UpdateProductMaterial,
    },
};

db_core::impl_repository!(
    ProductMaterialRepo,
    product_materials::Entity,
    product_materials::Model
);

pub struct ProductMaterialService {
    repo: ProductMaterialRepo,
}

impl ProductMaterialService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ProductMaterialRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateProductMaterial) -> BizResult<ProductMaterial> {
        let model = product_materials::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            material_type: Set(input.material_type),
            title: Set(input.title),
            version: Set(input.version),
            content: Set(input.content),
            status: Set(ProductMaterialStatus::Active.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ProductMaterial>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ProductMaterialFilter) -> BizResult<Vec<ProductMaterial>> {
        let mut query = self
            .repo
            .query()
            .filter(product_materials::Column::UserId.eq(filter.user_id))
            .filter(product_materials::Column::ProductId.eq(filter.product_id));

        if let Some(status) = filter.status {
            query = query.filter(product_materials::Column::Status.eq(status.as_str()));
        }

        if let Some(material_type) = filter.material_type {
            query = query.filter(product_materials::Column::MaterialType.eq(material_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(product_materials::Column::Title.contains(keyword.clone()))
                .add(product_materials::Column::Content.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(product_materials::Column::UpdatedAt);

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

    pub async fn list_versions(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        material_type: String,
    ) -> BizResult<Vec<ProductMaterial>> {
        let query = self
            .repo
            .query()
            .filter(product_materials::Column::UserId.eq(user_id))
            .filter(product_materials::Column::ProductId.eq(product_id))
            .filter(product_materials::Column::MaterialType.eq(material_type))
            .order_by_desc(product_materials::Column::Version);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn update(&self, input: UpdateProductMaterial) -> BizResult<ProductMaterial> {
        let model = product_materials::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            version: Set(input.version),
            content: Set(input.content),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: product_materials::Model) -> BizResult<ProductMaterial> {
        Ok(ProductMaterial {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            material_type: model.material_type,
            title: model.title,
            version: model.version,
            content: model.content,
            status: ProductMaterialStatus::try_from(model.status)?,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
