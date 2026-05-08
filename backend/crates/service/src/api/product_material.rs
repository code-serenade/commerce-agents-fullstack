use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    product_materials::{
        CreateProductMaterial, ProductMaterial, ProductMaterialFilter, ProductMaterialService,
        ProductMaterialStatus, UpdateProductMaterial,
    },
    products::{Product, ProductService, ProductStatus},
};
use serde_json::json;
use uuid::Uuid;

use crate::dto::product_material::{
    CreateProductMaterialRequest, DeleteProductMaterialRequest, GetProductMaterialRequest,
    ListProductMaterialsRequest, ProductMaterialResponse, UpdateProductMaterialRequest,
};

const DEFAULT_LIST_LIMIT: u64 = 200;

pub struct ProductMaterialApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    material_svc: ProductMaterialService,
}

impl ProductMaterialApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            material_svc: ProductMaterialService::new(db),
        }
    }

    pub async fn list_materials(
        &self,
        req: ListProductMaterialsRequest,
    ) -> BizResult<Vec<ProductMaterialResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let status = parse_optional_status(req.status)?;

        self.material_svc
            .list(ProductMaterialFilter {
                user_id,
                product_id,
                status: Some(status.unwrap_or(ProductMaterialStatus::Active)),
                material_type: normalize_optional(req.material_type),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_material)
            .collect()
    }

    pub async fn create_material(
        &self,
        req: CreateProductMaterialRequest,
    ) -> BizResult<ProductMaterialResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let material_type = normalize_required(req.material_type, "material_type")?;
        let version = self
            .next_version(user_id, product_id, material_type.clone())
            .await?;

        let material = self
            .material_svc
            .create(CreateProductMaterial {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                material_type,
                title: normalize_optional(req.title),
                version,
                content: normalize_optional(req.content).unwrap_or_default(),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_material(material)
    }

    pub async fn get_material(
        &self,
        req: GetProductMaterialRequest,
    ) -> BizResult<ProductMaterialResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let material_id = parse_uuid(&req.material_id, "material_id")?;
        let material = self
            .ensure_active_material_owner(user_id, product_id, material_id)
            .await?;
        Self::map_material(material)
    }

    pub async fn update_material(
        &self,
        req: UpdateProductMaterialRequest,
    ) -> BizResult<ProductMaterialResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let material_id = parse_uuid(&req.material_id, "material_id")?;
        let current = self
            .ensure_active_material_owner(user_id, product_id, material_id)
            .await?;

        let content_changed = req.content.is_some();
        let material = self
            .material_svc
            .update(UpdateProductMaterial {
                id: current.id,
                title: req.title.map(normalize_string).unwrap_or(current.title),
                version: if content_changed {
                    current.version + 1
                } else {
                    current.version
                },
                content: req
                    .content
                    .map(|value| value.trim().to_string())
                    .unwrap_or(current.content),
                status: current.status,
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;

        Self::map_material(material)
    }

    pub async fn delete_material(&self, req: DeleteProductMaterialRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let material_id = parse_uuid(&req.material_id, "material_id")?;
        let current = self
            .ensure_active_material_owner(user_id, product_id, material_id)
            .await?;

        self.material_svc
            .update(UpdateProductMaterial {
                id: current.id,
                title: current.title,
                version: current.version,
                content: current.content,
                status: ProductMaterialStatus::Archived,
                metadata: current.metadata,
            })
            .await?;

        Ok(())
    }

    pub async fn material_versions(
        &self,
        req: GetProductMaterialRequest,
    ) -> BizResult<Vec<ProductMaterialResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let material_id = parse_uuid(&req.material_id, "material_id")?;
        let material = self
            .ensure_material_owner(user_id, product_id, material_id)
            .await?;

        self.material_svc
            .list_versions(user_id, product_id, material.material_type)
            .await?
            .into_iter()
            .map(Self::map_material)
            .collect()
    }

    async fn next_version(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        material_type: String,
    ) -> BizResult<i32> {
        let current_max = self
            .material_svc
            .list_versions(user_id, product_id, material_type)
            .await?
            .into_iter()
            .map(|item| item.version)
            .max()
            .unwrap_or(0);
        Ok(current_max + 1)
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

    async fn ensure_material_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        material_id: Uuid,
    ) -> BizResult<ProductMaterial> {
        let material = self
            .material_svc
            .get_by_id(material_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_MATERIAL_NOT_FOUND,
                    "material not found".to_string(),
                )
            })?;
        if material.user_id != user_id || material.product_id != product_id {
            return Err(BizError::new(
                app_error::PRODUCT_MATERIAL_FORBIDDEN,
                "material does not belong to current product".to_string(),
            ));
        }
        Ok(material)
    }

    async fn ensure_active_material_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        material_id: Uuid,
    ) -> BizResult<ProductMaterial> {
        let material = self
            .ensure_material_owner(user_id, product_id, material_id)
            .await?;
        if material.status == ProductMaterialStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_MATERIAL_NOT_FOUND,
                "material not found".to_string(),
            ));
        }
        Ok(material)
    }

    fn map_material(item: ProductMaterial) -> BizResult<ProductMaterialResponse> {
        Ok(ProductMaterialResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.to_string(),
            material_type: item.material_type,
            title: item.title,
            version: item.version,
            content: item.content,
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
            app_error::PRODUCT_MATERIAL_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductMaterialStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "active" => Ok(ProductMaterialStatus::Active),
            "archived" => Ok(ProductMaterialStatus::Archived),
            _ => Err(BizError::new(
                app_error::PRODUCT_MATERIAL_REQUEST_INVALID,
                format!("invalid product material status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_MATERIAL_REQUEST_INVALID,
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
