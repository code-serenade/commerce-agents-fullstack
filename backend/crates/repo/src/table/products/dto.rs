use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Draft,
    Listing,
    Active,
    Archived,
}

impl ProductStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Listing => "listing",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProductStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "listing" => Ok(Self::Listing),
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid product status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for ProductStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProduct {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProduct {
    pub id: Uuid,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilter {
    pub user_id: Uuid,
    pub status: Option<ProductStatus>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}
