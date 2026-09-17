use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StudentLearningProbe {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub assistant_message_id: Option<Uuid>,
    pub answer_message_id: Option<Uuid>,
    pub framework_version: String,
    pub knowledge_code: String,
    pub status: String,
    pub question_text: String,
    pub reason_code: String,
    pub priority_score: f64,
    pub context_snapshot: Value,
    pub offered_at: Option<OffsetDateTime>,
    pub answered_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateStudentLearningProbe {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub assistant_message_id: Option<Uuid>,
    pub framework_version: String,
    pub knowledge_code: String,
    pub question_text: String,
    pub reason_code: String,
    pub priority_score: f64,
    pub context_snapshot: Value,
    pub offered_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct UpdateStudentLearningProbeStatus {
    pub id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct UpdateStudentLearningProbeContext {
    pub id: Uuid,
    pub context_snapshot: Value,
}

#[derive(Debug, Clone)]
pub struct MarkStudentLearningProbeAnswered {
    pub id: Uuid,
    pub answer_message_id: Uuid,
    pub answered_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct MarkStudentLearningProbeStructuredAnswer {
    pub id: Uuid,
    pub context_snapshot: Value,
    pub answered_at: OffsetDateTime,
}
