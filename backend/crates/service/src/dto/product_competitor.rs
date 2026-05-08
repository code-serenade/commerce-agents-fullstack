use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListProductCompetitorsRequest {
    pub user_id: String,
    pub product_id: String,
    pub status: Option<String>,
    pub platform: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductCompetitorRequest {
    pub user_id: String,
    pub product_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Option<Value>,
    pub metadata: Option<Value>,
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetProductCompetitorRequest {
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductCompetitorRequest {
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Option<Value>,
    pub metadata: Option<Value>,
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductCompetitorRequest {
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListCompetitorChangesRequest {
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: Option<String>,
    pub change_type: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCompetitorChangeRequest {
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: Option<String>,
    pub source_type: Option<String>,
    pub metadata: Option<Value>,
    pub detected_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductCompetitorResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Value,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_searched_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompetitorChangeResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
    pub competitor_name: Option<String>,
    pub competitor_platform: Option<String>,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: Option<String>,
    pub source_type: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub detected_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
