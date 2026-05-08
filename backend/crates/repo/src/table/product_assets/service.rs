use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::product_assets,
    table::product_assets::dto::{
        CreateProductAsset, ProductAsset, ProductAssetFilter, ProductAssetStatus,
        UpdateProductAsset,
    },
};

db_core::impl_repository!(
    ProductAssetRepo,
    product_assets::Entity,
    product_assets::Model
);

pub struct ProductAssetService {
    repo: ProductAssetRepo,
}

impl ProductAssetService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ProductAssetRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateProductAsset) -> BizResult<ProductAsset> {
        let model = product_assets::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            bucket: Set(input.bucket),
            object_key: Set(input.object_key),
            asset_type: Set(input.asset_type),
            file_name: Set(input.file_name),
            mime_type: Set(input.mime_type),
            file_size: Set(input.file_size),
            description: Set(input.description),
            status: Set(ProductAssetStatus::Active.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ProductAsset>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ProductAssetFilter) -> BizResult<Vec<ProductAsset>> {
        let mut query = self
            .repo
            .query()
            .filter(product_assets::Column::UserId.eq(filter.user_id))
            .filter(product_assets::Column::ProductId.eq(filter.product_id));

        if let Some(status) = filter.status {
            query = query.filter(product_assets::Column::Status.eq(status.as_str()));
        }

        if let Some(asset_type) = filter.asset_type {
            query = query.filter(product_assets::Column::AssetType.eq(asset_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(product_assets::Column::FileName.contains(keyword.clone()))
                .add(product_assets::Column::ObjectKey.contains(keyword.clone()))
                .add(product_assets::Column::Description.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(product_assets::Column::UpdatedAt);

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

    pub async fn update(&self, input: UpdateProductAsset) -> BizResult<ProductAsset> {
        let model = product_assets::ActiveModel {
            id: Unchanged(input.id),
            asset_type: Set(input.asset_type),
            file_name: Set(input.file_name),
            mime_type: Set(input.mime_type),
            file_size: Set(input.file_size),
            description: Set(input.description),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: product_assets::Model) -> BizResult<ProductAsset> {
        Ok(ProductAsset {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            bucket: model.bucket,
            object_key: model.object_key,
            asset_type: model.asset_type,
            file_name: model.file_name,
            mime_type: model.mime_type,
            file_size: model.file_size,
            description: model.description,
            status: ProductAssetStatus::try_from(model.status)?,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
