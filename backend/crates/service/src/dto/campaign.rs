use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListCampaignsRequest {
    pub user_id: String,
    pub status: Option<String>,
    pub product_id: Option<String>,
    pub campaign_type: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCampaignRequest {
    pub user_id: String,
    pub product_id: Option<String>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
    pub output: Option<Value>,
    pub action_metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetCampaignRequest {
    pub user_id: String,
    pub campaign_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCampaignRequest {
    pub user_id: String,
    pub campaign_id: String,
    pub product_id: Option<String>,
    pub name: Option<String>,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
    pub output: Option<Value>,
    pub action_metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteCampaignRequest {
    pub user_id: String,
    pub campaign_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignListResponse {
    pub campaigns: Vec<CampaignResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: Option<String>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub output: Value,
    pub action_metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
