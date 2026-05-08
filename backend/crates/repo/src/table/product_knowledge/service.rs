use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::product_knowledge,
    table::product_knowledge::dto::{
        CreateProductKnowledge, ProductKnowledge, ProductKnowledgeFilter, ProductKnowledgeStatus,
        UpdateProductKnowledge,
    },
};

db_core::impl_repository!(
    ProductKnowledgeRepo,
    product_knowledge::Entity,
    product_knowledge::Model
);

pub struct ProductKnowledgeService {
    repo: ProductKnowledgeRepo,
}

impl ProductKnowledgeService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ProductKnowledgeRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateProductKnowledge) -> BizResult<ProductKnowledge> {
        let model = product_knowledge::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            title: Set(input.title),
            content_type: Set(input.content_type),
            content: Set(input.content),
            source_type: Set(input.source_type),
            source_id: Set(input.source_id),
            confidence: Set(input.confidence),
            status: Set(ProductKnowledgeStatus::Active.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ProductKnowledge>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ProductKnowledgeFilter) -> BizResult<Vec<ProductKnowledge>> {
        let mut query = self
            .repo
            .query()
            .filter(product_knowledge::Column::UserId.eq(filter.user_id))
            .filter(product_knowledge::Column::ProductId.eq(filter.product_id));

        if let Some(status) = filter.status {
            query = query.filter(product_knowledge::Column::Status.eq(status.as_str()));
        }

        if let Some(content_type) = filter.content_type {
            query = query.filter(product_knowledge::Column::ContentType.eq(content_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(product_knowledge::Column::Title.contains(keyword.clone()))
                .add(product_knowledge::Column::Content.contains(keyword.clone()))
                .add(product_knowledge::Column::SourceType.contains(keyword.clone()))
                .add(product_knowledge::Column::SourceId.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query
            .order_by_desc(product_knowledge::Column::Confidence)
            .order_by_desc(product_knowledge::Column::UpdatedAt);

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

    pub async fn update(&self, input: UpdateProductKnowledge) -> BizResult<ProductKnowledge> {
        let model = product_knowledge::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            content_type: Set(input.content_type),
            content: Set(input.content),
            source_type: Set(input.source_type),
            source_id: Set(input.source_id),
            confidence: Set(input.confidence),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: product_knowledge::Model) -> BizResult<ProductKnowledge> {
        Ok(ProductKnowledge {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            title: model.title,
            content_type: model.content_type,
            content: model.content,
            source_type: model.source_type,
            source_id: model.source_id,
            confidence: model.confidence,
            status: ProductKnowledgeStatus::try_from(model.status)?,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
