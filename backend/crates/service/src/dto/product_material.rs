use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListProductMaterialsRequest {
    pub user_id: String,
    pub product_id: String,
    pub status: Option<String>,
    pub material_type: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductMaterialRequest {
    pub user_id: String,
    pub product_id: String,
    pub material_type: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetProductMaterialRequest {
    pub user_id: String,
    pub product_id: String,
    pub material_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductMaterialRequest {
    pub user_id: String,
    pub product_id: String,
    pub material_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductMaterialRequest {
    pub user_id: String,
    pub product_id: String,
    pub material_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductMaterialResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub material_type: String,
    pub title: Option<String>,
    pub version: i32,
    pub content: String,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
