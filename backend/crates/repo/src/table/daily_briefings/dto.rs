use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBriefing {
    pub id: Uuid,
    pub user_id: Uuid,
    pub briefing_date: String,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Value,
    pub product_summary: Value,
    pub alert_summary: Value,
    pub task_summary: Value,
    pub source: String,
    pub status: String,
    pub archived: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertDailyBriefing {
    pub id: Uuid,
    pub user_id: Uuid,
    pub briefing_date: String,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Value,
    pub product_summary: Value,
    pub alert_summary: Value,
    pub task_summary: Value,
    pub source: String,
    pub status: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDailyBriefing {
    pub id: Uuid,
    pub title: String,
    pub summary: Option<String>,
    pub sections: Value,
    pub product_summary: Value,
    pub alert_summary: Value,
    pub task_summary: Value,
    pub source: String,
    pub status: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DailyBriefingFilter {
    pub user_id: Uuid,
    pub archived: Option<bool>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
