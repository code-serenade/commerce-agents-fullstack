use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductAssetStatus {
    Active,
    Archived,
}

impl ProductAssetStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProductAssetStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid product asset status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for ProductAssetStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAsset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub asset_type: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub status: ProductAssetStatus,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductAsset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub asset_type: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductAsset {
    pub id: Uuid,
    pub asset_type: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub status: ProductAssetStatus,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProductAssetFilter {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub status: Option<ProductAssetStatus>,
    pub asset_type: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
}
