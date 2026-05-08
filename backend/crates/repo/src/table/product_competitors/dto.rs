use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductCompetitorStatus {
    Active,
    Archived,
}

impl ProductCompetitorStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProductCompetitorStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid product competitor status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for ProductCompetitorStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCompetitor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Value,
    pub status: ProductCompetitorStatus,
    pub metadata: Value,
    pub last_searched_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductCompetitor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Value,
    pub metadata: Value,
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductCompetitor {
    pub id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Value,
    pub status: ProductCompetitorStatus,
    pub metadata: Value,
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductCompetitorFilter {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub status: Option<ProductCompetitorStatus>,
    pub platform: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}
