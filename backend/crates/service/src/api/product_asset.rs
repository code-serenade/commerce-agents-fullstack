use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    product_assets::{
        CreateProductAsset, ProductAsset, ProductAssetFilter, ProductAssetService,
        ProductAssetStatus, UpdateProductAsset,
    },
    products::{Product, ProductService, ProductStatus},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::storage_reference::{RetainStorageReferenceRequest, StorageReferenceApi},
    dto::product_asset::{
        DeleteProductAssetRequest, GetProductAssetRequest, ListProductAssetsRequest,
        ProductAssetResponse, RegisterProductAssetRequest, UpdateProductAssetRequest,
    },
};

const DEFAULT_LIST_LIMIT: u64 = 200;
const DEFAULT_ASSET_TYPE: &str = "docs";

pub struct ProductAssetApi {
    db: DbContext,
    app_user_svc: AppUserService,
    product_svc: ProductService,
    asset_svc: ProductAssetService,
}

impl ProductAssetApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            asset_svc: ProductAssetService::new(db),
        }
    }

    pub async fn list_assets(
        &self,
        req: ListProductAssetsRequest,
    ) -> BizResult<Vec<ProductAssetResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let status = parse_optional_status(req.status)?;

        self.asset_svc
            .list(ProductAssetFilter {
                user_id,
                product_id,
                status: Some(status.unwrap_or(ProductAssetStatus::Active)),
                asset_type: normalize_optional(req.asset_type),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_asset)
            .collect()
    }

    pub async fn register_asset(
        &self,
        req: RegisterProductAssetRequest,
    ) -> BizResult<ProductAssetResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        ensure_product_asset_object_key(&req.object_key, &user_id)?;

        let bucket = normalize_required(req.bucket, "bucket")?;
        let object_key = normalize_required(req.object_key, "object_key")?;
        if let Some(existing) = self
            .asset_svc
            .get_active_by_product_object(product_id, &bucket, &object_key)
            .await?
        {
            StorageReferenceApi::new(self.db.clone())
                .retain(RetainStorageReferenceRequest {
                    user_id,
                    bucket: existing.bucket.clone(),
                    object_key: existing.object_key.clone(),
                    sha256: None,
                    mime_type: existing.mime_type.clone(),
                    file_size: existing.file_size,
                    source_type: "product_asset".to_string(),
                    source_id: existing.id,
                    metadata: json!({}),
                })
                .await?;
            return Self::map_asset(existing);
        }

        let tx = self.db.begin().await?;
        let result: BizResult<ProductAssetResponse> = async {
            let api = ProductAssetApi::new(tx.clone());
            let asset = api
                .asset_svc
                .create(CreateProductAsset {
                    id: Uuid::new_v4(),
                    user_id,
                    product_id,
                    bucket,
                    object_key,
                    asset_type: normalize_optional(req.asset_type)
                        .unwrap_or_else(|| DEFAULT_ASSET_TYPE.to_string()),
                    file_name: normalize_optional(req.file_name),
                    mime_type: normalize_optional(req.mime_type),
                    file_size: req.file_size,
                    description: normalize_optional(req.description),
                    metadata: req.metadata.unwrap_or_else(|| json!({})),
                })
                .await?;

            StorageReferenceApi::new(tx.clone())
                .retain_in_current_context(RetainStorageReferenceRequest {
                    user_id,
                    bucket: asset.bucket.clone(),
                    object_key: asset.object_key.clone(),
                    sha256: None,
                    mime_type: asset.mime_type.clone(),
                    file_size: asset.file_size,
                    source_type: "product_asset".to_string(),
                    source_id: asset.id,
                    metadata: json!({}),
                })
                .await?;

            Self::map_asset(asset)
        }
        .await;

        commit_or_rollback(tx, result).await
    }

    pub async fn get_asset(&self, req: GetProductAssetRequest) -> BizResult<ProductAssetResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let asset_id = parse_uuid(&req.asset_id, "asset_id")?;
        let asset = self
            .ensure_active_asset_owner(user_id, product_id, asset_id)
            .await?;
        Self::map_asset(asset)
    }

    pub async fn update_asset(
        &self,
        req: UpdateProductAssetRequest,
    ) -> BizResult<ProductAssetResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let asset_id = parse_uuid(&req.asset_id, "asset_id")?;
        let current = self
            .ensure_active_asset_owner(user_id, product_id, asset_id)
            .await?;

        let asset = self
            .asset_svc
            .update(UpdateProductAsset {
                id: current.id,
                asset_type: normalize_optional(req.asset_type).unwrap_or(current.asset_type),
                file_name: req
                    .file_name
                    .map(normalize_string)
                    .unwrap_or(current.file_name),
                mime_type: req
                    .mime_type
                    .map(normalize_string)
                    .unwrap_or(current.mime_type),
                file_size: req.file_size.or(current.file_size),
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                status: current.status,
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;

        Self::map_asset(asset)
    }

    pub async fn delete_asset(&self, req: DeleteProductAssetRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let asset_id = parse_uuid(&req.asset_id, "asset_id")?;
        let current = self
            .ensure_active_asset_owner(user_id, product_id, asset_id)
            .await?;

        let tx = self.db.begin().await?;
        let result: BizResult<()> = async {
            let api = ProductAssetApi::new(tx.clone());
            StorageReferenceApi::new(tx.clone())
                .release_in_current_context("product_asset", current.id)
                .await?;
            api.asset_svc
                .update(UpdateProductAsset {
                    id: current.id,
                    asset_type: current.asset_type,
                    file_name: current.file_name,
                    mime_type: current.mime_type,
                    file_size: current.file_size,
                    description: current.description,
                    status: ProductAssetStatus::Archived,
                    metadata: current.metadata,
                })
                .await?;

            Ok(())
        }
        .await;

        commit_or_rollback(tx, result).await
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

    async fn ensure_asset_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        asset_id: Uuid,
    ) -> BizResult<ProductAsset> {
        let asset = self.asset_svc.get_by_id(asset_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::PRODUCT_ASSET_NOT_FOUND,
                "asset not found".to_string(),
            )
        })?;
        if asset.user_id != user_id || asset.product_id != product_id {
            return Err(BizError::new(
                app_error::PRODUCT_ASSET_FORBIDDEN,
                "asset does not belong to current product".to_string(),
            ));
        }
        Ok(asset)
    }

    async fn ensure_active_asset_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        asset_id: Uuid,
    ) -> BizResult<ProductAsset> {
        let asset = self
            .ensure_asset_owner(user_id, product_id, asset_id)
            .await?;
        if asset.status == ProductAssetStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_ASSET_NOT_FOUND,
                "asset not found".to_string(),
            ));
        }
        Ok(asset)
    }

    fn map_asset(item: ProductAsset) -> BizResult<ProductAssetResponse> {
        Ok(ProductAssetResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.to_string(),
            bucket: item.bucket,
            object_key: item.object_key,
            asset_type: item.asset_type,
            file_name: item.file_name,
            mime_type: item.mime_type,
            file_size: item.file_size,
            description: item.description,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

pub fn product_asset_object_key(user_scope: &str, product_id: &str, ext: &str) -> String {
    format!(
        "products/{user_scope}/{product_id}/assets/{}.{}",
        Uuid::new_v4().as_simple(),
        ext
    )
}

pub fn expected_product_asset_prefix(user_scope: &str, product_id: &str) -> String {
    format!("products/{user_scope}/{product_id}/assets/")
}

fn ensure_product_asset_object_key(object_key: &str, user_id: &Uuid) -> BizResult<()> {
    storage::key::validate_object_key(object_key)
        .map_err(|err| BizError::new(app_error::PRODUCT_ASSET_REQUEST_INVALID, err.to_string()))?;

    let user_scope = storage::key::user_scope_key(&user_id.to_string(), None);
    for prefix in [
        format!("users/{user_scope}/docs/"),
        format!("users/{user_scope}/images/"),
    ] {
        if object_key.starts_with(&prefix) {
            return Ok(());
        }
    }

    Err(BizError::new(
        app_error::PRODUCT_ASSET_REQUEST_INVALID,
        "invalid product asset object key".to_string(),
    ))
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PRODUCT_ASSET_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductAssetStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "active" => Ok(ProductAssetStatus::Active),
            "archived" => Ok(ProductAssetStatus::Archived),
            _ => Err(BizError::new(
                app_error::PRODUCT_ASSET_REQUEST_INVALID,
                format!("invalid product asset status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_ASSET_REQUEST_INVALID,
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

async fn commit_or_rollback<T>(tx: DbContext, result: BizResult<T>) -> BizResult<T> {
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
