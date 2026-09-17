use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;
#[derive(Debug, Clone)]
pub struct StudentLearningProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub framework_version: String,
    pub mastery_map: Value,
    pub engagement_summary: Value,
    pub coverage_rate: f64,
    pub analysis_status: String,
    pub last_analyzed_sequence: i64,
    pub last_analyzed_at: Option<OffsetDateTime>,
    pub last_error_code: Option<String>,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
#[derive(Debug, Clone)]
pub struct CreateStudentLearningProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub framework_version: String,
}

#[derive(Debug, Clone)]
pub struct UpdateStudentLearningProfileSettings {
    pub id: Uuid,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateStudentLearningAnalysis {
    pub id: Uuid,
    pub mastery_map: serde_json::Value,
    pub engagement_summary: serde_json::Value,
    pub coverage_rate: f64,
    pub analysis_status: String,
    pub last_analyzed_sequence: i64,
    pub last_analyzed_at: OffsetDateTime,
    pub last_error_code: Option<String>,
}
