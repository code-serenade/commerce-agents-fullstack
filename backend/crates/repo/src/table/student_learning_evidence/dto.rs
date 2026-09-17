use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateStudentLearningEvidence {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub source_message_id: Option<Uuid>,
    pub probe_id: Option<Uuid>,
    pub framework_version: String,
    pub knowledge_code: String,
    pub evidence_type: String,
    pub direction: String,
    pub strength: f64,
    pub confidence: f64,
    pub summary: String,
    pub rubric_hits: Value,
    pub analyzer_version: String,
    pub created_at: OffsetDateTime,
}
