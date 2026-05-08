use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetric {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub metric_date: String,
    pub metric_key: String,
    pub metric_value: f64,
    pub dimension_key: String,
    pub dimension_value: String,
    pub source: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertStoreMetric {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub metric_date: String,
    pub metric_key: String,
    pub metric_value: f64,
    pub dimension_key: String,
    pub dimension_value: String,
    pub source: String,
}

#[derive(Debug, Clone, Default)]
pub struct StoreMetricFilter {
    pub user_id: Uuid,
    pub platform: Option<String>,
    pub metric_key: Option<String>,
    pub metric_date_from: Option<String>,
    pub metric_date_to: Option<String>,
    pub source: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetricAggregate {
    pub platform: String,
    pub metric_key: String,
    pub total: f64,
}
