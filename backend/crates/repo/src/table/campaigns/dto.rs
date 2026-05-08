use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Ended,
    Archived,
}

impl CampaignStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Ended => "ended",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for CampaignStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "ended" => Ok(Self::Ended),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid campaign status: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for CampaignStatus {
    type Error = BizError;

    fn try_from(value: String) -> BizResult<Self> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: CampaignStatus,
    pub metadata: Value,
    pub output: Value,
    pub action_metadata: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCampaign {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: CampaignStatus,
    pub metadata: Value,
    pub output: Value,
    pub action_metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCampaign {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: CampaignStatus,
    pub metadata: Value,
    pub output: Value,
    pub action_metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct CampaignFilter {
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub status: Option<CampaignStatus>,
    pub campaign_type: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
