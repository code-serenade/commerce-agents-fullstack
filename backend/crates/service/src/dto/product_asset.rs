use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListProductAssetsRequest {
    pub user_id: String,
    pub product_id: String,
    pub status: Option<String>,
    pub asset_type: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterProductAssetRequest {
    pub user_id: String,
    pub product_id: String,
    pub bucket: String,
    pub object_key: String,
    pub asset_type: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductAssetRequest {
    pub user_id: String,
    pub product_id: String,
    pub asset_id: String,
    pub asset_type: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductAssetRequest {
    pub user_id: String,
    pub product_id: String,
    pub asset_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetProductAssetRequest {
    pub user_id: String,
    pub product_id: String,
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductAssetResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub bucket: String,
    pub object_key: String,
    pub asset_type: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
