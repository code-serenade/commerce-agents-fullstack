use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListProductsRequest {
    pub user_id: String,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductRequest {
    pub user_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetProductRequest {
    pub user_id: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductRequest {
    pub user_id: String,
    pub product_id: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductRequest {
    pub user_id: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub status: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductDetailResponse {
    pub product: ProductResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductDashboardResponse {
    pub total: u64,
    pub active: u64,
    pub archived: u64,
}
