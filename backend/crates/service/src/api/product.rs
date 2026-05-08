use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    products::{
        CreateProduct, Product, ProductFilter, ProductService, ProductStatus, UpdateProduct,
    },
};
use serde_json::json;
use uuid::Uuid;

use crate::dto::product::{
    CreateProductRequest, DeleteProductRequest, GetProductRequest, ListProductsRequest,
    ProductDashboardResponse, ProductDetailResponse, ProductResponse, UpdateProductRequest,
};

const DEFAULT_LIST_LIMIT: u64 = 100;

pub struct ProductApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
}

impl ProductApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db),
        }
    }

    pub async fn list_products(&self, req: ListProductsRequest) -> BizResult<Vec<ProductResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let status = parse_optional_status(req.status)?;

        self.product_svc
            .list(ProductFilter {
                user_id,
                status: Some(status.unwrap_or(ProductStatus::Active)),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_product)
            .collect()
    }

    pub async fn create_product(&self, req: CreateProductRequest) -> BizResult<ProductResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let name = normalize_required(req.name, "name")?;

        let product = self
            .product_svc
            .create(CreateProduct {
                id: Uuid::new_v4(),
                user_id,
                name,
                platform: normalize_optional(req.platform),
                external_id: normalize_optional(req.external_id),
                category: normalize_optional(req.category),
                brand: normalize_optional(req.brand),
                description: normalize_optional(req.description),
                price: normalize_optional(req.price),
                currency: normalize_optional(req.currency),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_product(product)
    }

    pub async fn get_product(&self, req: GetProductRequest) -> BizResult<ProductDetailResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let product = self.ensure_product_owner(user_id, product_id).await?;

        Ok(ProductDetailResponse {
            product: Self::map_product(product)?,
        })
    }

    pub async fn update_product(&self, req: UpdateProductRequest) -> BizResult<ProductResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let current = self
            .ensure_active_product_owner(user_id, product_id)
            .await?;

        let product = self
            .product_svc
            .update(UpdateProduct {
                id: current.id,
                name: normalize_optional(req.name).unwrap_or(current.name),
                platform: req
                    .platform
                    .map(normalize_string)
                    .unwrap_or(current.platform),
                external_id: req
                    .external_id
                    .map(normalize_string)
                    .unwrap_or(current.external_id),
                category: req
                    .category
                    .map(normalize_string)
                    .unwrap_or(current.category),
                brand: req.brand.map(normalize_string).unwrap_or(current.brand),
                status: current.status,
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                price: req.price.map(normalize_string).unwrap_or(current.price),
                currency: req
                    .currency
                    .map(normalize_string)
                    .unwrap_or(current.currency),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;

        Self::map_product(product)
    }

    pub async fn delete_product(&self, req: DeleteProductRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let current = self
            .ensure_active_product_owner(user_id, product_id)
            .await?;

        self.product_svc
            .update(UpdateProduct {
                id: current.id,
                name: current.name,
                platform: current.platform,
                external_id: current.external_id,
                category: current.category,
                brand: current.brand,
                status: ProductStatus::Archived,
                description: current.description,
                price: current.price,
                currency: current.currency,
                metadata: current.metadata,
            })
            .await?;

        Ok(())
    }

    pub async fn dashboard(&self, user_id: String) -> BizResult<ProductDashboardResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let active = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Active)
            .await?;
        let archived = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Archived)
            .await?;

        Ok(ProductDashboardResponse {
            total: active + archived,
            active,
            archived,
        })
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

    fn map_product(item: Product) -> BizResult<ProductResponse> {
        Ok(ProductResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            name: item.name,
            platform: item.platform,
            external_id: item.external_id,
            category: item.category,
            brand: item.brand,
            status: item.status.as_str().to_string(),
            description: item.description,
            price: item.price,
            currency: item.currency,
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
            app_error::PRODUCT_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "active" => Ok(ProductStatus::Active),
            "archived" => Ok(ProductStatus::Archived),
            _ => Err(BizError::new(
                app_error::PRODUCT_REQUEST_INVALID,
                format!("invalid product status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
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
