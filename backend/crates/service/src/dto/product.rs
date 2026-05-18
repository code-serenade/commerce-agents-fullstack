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
    pub status: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct GetProductLifecycleRequest {
    pub user_id: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitionProductLifecycleRequest {
    pub user_id: String,
    pub product_id: String,
    pub target_status: String,
    pub reason: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
    pub draft: u64,
    pub listing: u64,
    pub active: u64,
    pub archived: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLifecycleOptionsResponse {
    pub statuses: Vec<ProductLifecycleStatusResponse>,
    pub transitions: Vec<ProductLifecycleTransitionResponse>,
    pub agent_context: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLifecycleStatusResponse {
    pub status: String,
    pub label: String,
    pub description: String,
    pub terminal: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLifecycleTransitionResponse {
    pub from: String,
    pub to: String,
    pub label: String,
    pub requires_reason: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLifecycleResponse {
    pub product: ProductResponse,
    pub current_status: String,
    pub allowed_next_statuses: Vec<String>,
    pub transitions: Vec<ProductLifecycleTransitionResponse>,
    pub warnings: Vec<String>,
    pub agent_context: Value,
}
