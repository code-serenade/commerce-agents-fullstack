use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorChange {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub competitor_id: Uuid,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: Option<String>,
    pub source_type: String,
    pub metadata: Value,
    pub detected_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCompetitorChange {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub competitor_id: Uuid,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: Option<String>,
    pub source_type: String,
    pub metadata: Value,
    pub detected_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct CompetitorChangeFilter {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub competitor_id: Option<Uuid>,
    pub change_type: Option<String>,
    pub limit: Option<u64>,
}
