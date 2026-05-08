use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListProductKnowledgeRequest {
    pub user_id: String,
    pub product_id: String,
    pub status: Option<String>,
    pub content_type: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductKnowledgeRequest {
    pub user_id: String,
    pub product_id: String,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub content: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub confidence: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductKnowledgeRequest {
    pub user_id: String,
    pub product_id: String,
    pub knowledge_id: String,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub content: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub confidence: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductKnowledgeRequest {
    pub user_id: String,
    pub product_id: String,
    pub knowledge_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductKnowledgeResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub title: Option<String>,
    pub content_type: String,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: f64,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
