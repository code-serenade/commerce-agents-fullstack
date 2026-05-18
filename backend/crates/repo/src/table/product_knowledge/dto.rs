use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKnowledgeStatus {
    Active,
    Archived,
}

impl ProductKnowledgeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProductKnowledgeStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid product knowledge status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for ProductKnowledgeStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductKnowledge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub title: Option<String>,
    pub content_type: String,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: f64,
    pub status: ProductKnowledgeStatus,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductKnowledge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub title: Option<String>,
    pub content_type: String,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductKnowledge {
    pub id: Uuid,
    pub title: Option<String>,
    pub content_type: String,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: f64,
    pub status: ProductKnowledgeStatus,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProductKnowledgeFilter {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub status: Option<ProductKnowledgeStatus>,
    pub content_type: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductKnowledgeSearchFilter {
    pub user_id: Uuid,
    pub status: Option<ProductKnowledgeStatus>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}
