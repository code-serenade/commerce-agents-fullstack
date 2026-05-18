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
pub struct GetProductCompetitorAnalysisRequest {
    pub user_id: String,
    pub product_id: String,
    pub status: Option<String>,
    pub platform: Option<String>,
    pub change_limit: Option<u64>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorAnalysisResponse {
    pub product: ProductCompetitorAnalysisProductResponse,
    pub readiness: String,
    pub summary: String,
    pub counts: ProductCompetitorAnalysisCounts,
    pub price: ProductCompetitorPriceAnalysisResponse,
    pub rating: ProductCompetitorRatingAnalysisResponse,
    pub platforms: Vec<ProductCompetitorPlatformAnalysisResponse>,
    pub top_competitors: Vec<ProductCompetitorResponse>,
    pub recent_changes: Vec<CompetitorChangeResponse>,
    pub refs: Vec<ProductCompetitorAnalysisRefResponse>,
    pub signals: Vec<String>,
    pub warnings: Vec<String>,
    pub agent_context: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorAnalysisProductResponse {
    pub id: String,
    pub name: String,
    pub platform: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub price: Option<String>,
    pub numeric_price: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorAnalysisCounts {
    pub competitors: usize,
    pub priced_competitors: usize,
    pub rated_competitors: usize,
    pub monthly_sales_competitors: usize,
    pub recent_changes: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorPriceAnalysisResponse {
    pub product_price: Option<f64>,
    pub min_competitor_price: Option<f64>,
    pub max_competitor_price: Option<f64>,
    pub avg_competitor_price: Option<f64>,
    pub price_position: Option<String>,
    pub gap_to_avg: Option<f64>,
    pub gap_percent_to_avg: Option<f64>,
    pub lower_priced_competitors: usize,
    pub higher_priced_competitors: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorRatingAnalysisResponse {
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
    pub avg_rating: Option<f64>,
    pub rated_competitors: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorPlatformAnalysisResponse {
    pub platform: String,
    pub competitors: usize,
    pub avg_price: Option<f64>,
    pub avg_rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorAnalysisRefResponse {
    pub ref_type: String,
    pub id: String,
    pub title: String,
    pub ref_key: String,
}
