use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    product_knowledge::{
        CreateProductKnowledge, ProductKnowledge, ProductKnowledgeFilter, ProductKnowledgeService,
        ProductKnowledgeStatus, UpdateProductKnowledge,
    },
    products::{Product, ProductService, ProductStatus},
};
use serde_json::json;
use uuid::Uuid;

use crate::dto::product_knowledge::{
    CreateProductKnowledgeRequest, DeleteProductKnowledgeRequest, ListProductKnowledgeRequest,
    ProductKnowledgeResponse, UpdateProductKnowledgeRequest,
};

const DEFAULT_LIST_LIMIT: u64 = 200;
const DEFAULT_CONTENT_TYPE: &str = "text";
const DEFAULT_SOURCE_TYPE: &str = "manual";
const DEFAULT_CONFIDENCE: f64 = 0.8;

pub struct ProductKnowledgeApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    knowledge_svc: ProductKnowledgeService,
}

impl ProductKnowledgeApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            knowledge_svc: ProductKnowledgeService::new(db),
        }
    }

    pub async fn list_knowledge(
        &self,
        req: ListProductKnowledgeRequest,
    ) -> BizResult<Vec<ProductKnowledgeResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;

        let status = parse_optional_status(req.status)?;
        self.knowledge_svc
            .list(ProductKnowledgeFilter {
                user_id,
                product_id,
                status: Some(status.unwrap_or(ProductKnowledgeStatus::Active)),
                content_type: normalize_optional(req.content_type),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_knowledge)
            .collect()
    }

    pub async fn create_knowledge(
        &self,
        req: CreateProductKnowledgeRequest,
    ) -> BizResult<ProductKnowledgeResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let content = normalize_required(req.content, "content")?;
        let confidence = normalize_confidence(req.confidence.unwrap_or(DEFAULT_CONFIDENCE))?;

        let item = self
            .knowledge_svc
            .create(CreateProductKnowledge {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                title: normalize_optional(req.title),
                content_type: normalize_optional(req.content_type)
                    .unwrap_or_else(|| DEFAULT_CONTENT_TYPE.to_string()),
                content,
                source_type: normalize_optional(req.source_type)
                    .unwrap_or_else(|| DEFAULT_SOURCE_TYPE.to_string()),
                source_id: normalize_optional(req.source_id),
                confidence,
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_knowledge(item)
    }

    pub async fn update_knowledge(
        &self,
        req: UpdateProductKnowledgeRequest,
    ) -> BizResult<ProductKnowledgeResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let knowledge_id = parse_uuid(&req.knowledge_id, "knowledge_id")?;
        let current = self
            .ensure_active_knowledge_owner(user_id, product_id, knowledge_id)
            .await?;

        let item = self
            .knowledge_svc
            .update(UpdateProductKnowledge {
                id: current.id,
                title: req.title.map(normalize_string).unwrap_or(current.title),
                content_type: normalize_optional(req.content_type).unwrap_or(current.content_type),
                content: match req.content {
                    Some(content) => normalize_required(content, "content")?,
                    None => current.content,
                },
                source_type: normalize_optional(req.source_type).unwrap_or(current.source_type),
                source_id: req
                    .source_id
                    .map(normalize_string)
                    .unwrap_or(current.source_id),
                confidence: normalize_confidence(req.confidence.unwrap_or(current.confidence))?,
                status: current.status,
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;

        Self::map_knowledge(item)
    }

    pub async fn delete_knowledge(&self, req: DeleteProductKnowledgeRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let knowledge_id = parse_uuid(&req.knowledge_id, "knowledge_id")?;
        let current = self
            .ensure_active_knowledge_owner(user_id, product_id, knowledge_id)
            .await?;

        self.knowledge_svc
            .update(UpdateProductKnowledge {
                id: current.id,
                title: current.title,
                content_type: current.content_type,
                content: current.content,
                source_type: current.source_type,
                source_id: current.source_id,
                confidence: current.confidence,
                status: ProductKnowledgeStatus::Archived,
                metadata: current.metadata,
            })
            .await?;

        Ok(())
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_product_owner(&self, user_id: Uuid, product_id: Uuid) -> BizResult<Product> {
        let product = self
            .product_svc
            .get_by_id(product_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_NOT_FOUND,
                    "product not found".to_string(),
                )
            })?;
        if product.user_id != user_id {
            return Err(BizError::new(
                app_error::PRODUCT_FORBIDDEN,
                "product does not belong to current user".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_active_product_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> BizResult<Product> {
        let product = self.ensure_product_owner(user_id, product_id).await?;
        if product.status == ProductStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_NOT_FOUND,
                "product not found".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_knowledge_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        knowledge_id: Uuid,
    ) -> BizResult<ProductKnowledge> {
        let item = self
            .knowledge_svc
            .get_by_id(knowledge_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_KNOWLEDGE_NOT_FOUND,
                    "product knowledge not found".to_string(),
                )
            })?;
        if item.user_id != user_id || item.product_id != product_id {
            return Err(BizError::new(
                app_error::PRODUCT_KNOWLEDGE_FORBIDDEN,
                "product knowledge does not belong to current product".to_string(),
            ));
        }
        Ok(item)
    }

    async fn ensure_active_knowledge_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        knowledge_id: Uuid,
    ) -> BizResult<ProductKnowledge> {
        let item = self
            .ensure_knowledge_owner(user_id, product_id, knowledge_id)
            .await?;
        if item.status == ProductKnowledgeStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_KNOWLEDGE_NOT_FOUND,
                "product knowledge not found".to_string(),
            ));
        }
        Ok(item)
    }

    fn map_knowledge(item: ProductKnowledge) -> BizResult<ProductKnowledgeResponse> {
        Ok(ProductKnowledgeResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.to_string(),
            title: item.title,
            content_type: item.content_type,
            content: item.content,
            source_type: item.source_type,
            source_id: item.source_id,
            confidence: item.confidence,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PRODUCT_KNOWLEDGE_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductKnowledgeStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "active" => Ok(ProductKnowledgeStatus::Active),
            "archived" => Ok(ProductKnowledgeStatus::Archived),
            _ => Err(BizError::new(
                app_error::PRODUCT_KNOWLEDGE_REQUEST_INVALID,
                format!("invalid product knowledge status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_confidence(value: f64) -> BizResult<f64> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(BizError::new(
            app_error::PRODUCT_KNOWLEDGE_REQUEST_INVALID,
            "confidence must be between 0 and 1".to_string(),
        ));
    }
    Ok(value)
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_KNOWLEDGE_REQUEST_INVALID,
            format!("{label} cannot be empty"),
        ));
    }
    Ok(value)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(normalize_string)
}

fn normalize_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}
