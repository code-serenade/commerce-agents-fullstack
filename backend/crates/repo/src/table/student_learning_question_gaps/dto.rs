use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StudentLearningQuestionGap {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub assistant_message_id: Uuid,
    pub framework_version: String,
    pub selector_version: String,
    pub knowledge_code: String,
    pub reason_code: String,
    pub topic_signals: Value,
    pub scope_signals: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateStudentLearningQuestionGap {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub assistant_message_id: Uuid,
    pub framework_version: String,
    pub selector_version: String,
    pub knowledge_code: String,
    pub reason_code: String,
    pub topic_signals: Value,
    pub scope_signals: Value,
}
