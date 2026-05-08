use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductMaterialStatus {
    Active,
    Archived,
}

impl ProductMaterialStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProductMaterialStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid product material status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for ProductMaterialStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMaterial {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub material_type: String,
    pub title: Option<String>,
    pub version: i32,
    pub content: String,
    pub status: ProductMaterialStatus,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductMaterial {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub material_type: String,
    pub title: Option<String>,
    pub version: i32,
    pub content: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductMaterial {
    pub id: Uuid,
    pub title: Option<String>,
    pub version: i32,
    pub content: String,
    pub status: ProductMaterialStatus,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProductMaterialFilter {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub status: Option<ProductMaterialStatus>,
    pub material_type: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}
