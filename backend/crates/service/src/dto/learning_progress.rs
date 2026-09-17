use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProfileResponse {
    pub framework_version: String,
    pub mastery_map: Value,
    pub engagement_summary: Value,
    pub coverage_rate: f64,
    pub assessment_summary: LearningAssessmentSummary,
    pub assessment_timeline: Vec<LearningAssessmentTimelinePoint>,
    pub domain_assessments: Vec<LearningDomainAssessment>,
    pub analysis_status: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_analyzed_at: Option<OffsetDateTime>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LearningDomainAssessment {
    pub knowledge_code: String,
    pub status_code: String,
    pub reason_code: String,
    pub observed: bool,
    pub observation_count: u64,
    pub assessment_count: u64,
    pub distinct_question_count: usize,
    pub assessment_angle_count: usize,
    pub correct_count: u64,
    pub partial_count: u64,
    pub incorrect_count: u64,
    pub latest_outcome: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub latest_assessed_at: Option<OffsetDateTime>,
    pub retention_verified: bool,
    pub missing_evidence_codes: Vec<String>,
    pub next_action_code: String,
    pub recent_evidence: Vec<LearningDomainEvidenceSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LearningDomainEvidenceSummary {
    pub evidence_id: String,
    pub conversation_id: String,
    pub question_id: Option<String>,
    pub question_text: Option<String>,
    pub selected_choice_text: Option<String>,
    pub outcome: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LearningAssessmentTimelinePoint {
    pub period_start: String,
    pub assessed_knowledge_count: usize,
    pub demonstrated_knowledge_count: usize,
    pub retained_knowledge_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LearningAssessmentSummary {
    pub assessed_knowledge_count: usize,
    pub assessment_count: u64,
    pub distinct_assessment_question_count: usize,
    pub multi_angle_knowledge_count: usize,
    pub demonstrated_knowledge_count: usize,
    pub retained_knowledge_count: usize,
    pub correct_count: u64,
    pub partial_count: u64,
    pub incorrect_count: u64,
    pub needs_attention_knowledge_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLearningSettingsRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LearningEvidenceRange {
    #[default]
    Recent,
    Days30,
    Days90,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeResponse {
    pub id: String,
    pub question_id: Option<String>,
    pub question_version: Option<u32>,
    pub question_type: String,
    pub choices: Vec<LearningProbeChoice>,
    pub engaged: bool,
    pub knowledge_code: String,
    pub question_text: String,
    pub reason_code: String,
    pub interaction_mode: String,
    pub reward_code: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningProbeChoice {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeStatusResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeSourceLabel {
    pub source_id: String,
    pub title: String,
    pub publisher: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeResultResponse {
    pub id: String,
    pub status: String,
    pub can_continue: bool,
    pub outcome: Option<String>,
    pub score: Option<f64>,
    pub feedback_code: Option<String>,
    pub selected_choice_id: Option<String>,
    pub selected_choice_text: Option<String>,
    pub correct_choice_text: Option<String>,
    pub answer_comparison_code: Option<String>,
    pub knowledge_points: Vec<String>,
    pub demonstrated_points: Vec<String>,
    pub next_point: Option<String>,
    pub reference_explanation: Option<String>,
    pub source_labels: Vec<LearningProbeSourceLabel>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeHistoryEntry {
    pub probe: LearningProbeResponse,
    pub result: LearningProbeResultResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningAdminSortBy {
    CoverageRate,
    ObservedKnowledgeCount,
    LastAnalyzedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningSortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Default)]
pub struct ListLearningAdminStudentsRequest {
    pub page: u64,
    pub page_size: u64,
    pub keyword: Option<String>,
    pub sort_by: Option<LearningAdminSortBy>,
    pub sort_order: Option<LearningSortOrder>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningAdminStudentRow {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: String,
    pub team_name: String,
    pub coverage_rate: f64,
    pub observed_knowledge_count: usize,
    pub mastery_status: String,
    pub assessment_summary: LearningAssessmentSummary,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_analyzed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningDomainMastery {
    pub knowledge_code: String,
    pub status: String,
    pub strength: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEvidenceEntry {
    pub id: String,
    pub conversation_id: String,
    pub probe_id: Option<String>,
    pub question_text: Option<String>,
    pub selected_choice_id: Option<String>,
    pub selected_choice_text: Option<String>,
    pub knowledge_code: String,
    pub evidence_type: String,
    pub direction: String,
    pub strength: f64,
    pub summary: String,
    pub assessment_outcome: Option<String>,
    pub question_id: Option<String>,
    pub rubric_version: Option<u32>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningAdminStudentDetail {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: String,
    pub team_name: String,
    pub coverage_rate: f64,
    pub observed_knowledge_count: usize,
    pub mastery_status: String,
    pub assessment_summary: LearningAssessmentSummary,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_analyzed_at: Option<OffsetDateTime>,
    pub mastery_summary: Vec<LearningDomainMastery>,
    pub evidence_summary: Vec<LearningEvidenceEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningAdminMetrics {
    pub total: u64,
    pub enabled: u64,
    pub observed: u64,
    pub avg_coverage_rate: f64,
    pub assessed: u64,
    pub demonstrated: u64,
    pub retained: u64,
    pub needs_attention: u64,
    pub probe_pilot: LearningProbePilotMetrics,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbePilotMetrics {
    pub question_bank_version: String,
    pub question_count: u64,
    pub assessment_angle_count: u64,
    pub single_question_angle_count: u64,
    pub production_approved_count: u64,
    pub curriculum_review_pending_count: u64,
    pub earliest_source_valid_through: Option<String>,
    pub issued_count: u64,
    pub active_count: u64,
    pub answered_count: u64,
    pub skipped_count: u64,
    pub expired_count: u64,
    pub superseded_count: u64,
    pub continuation_count: u64,
    pub correct_count: u64,
    pub partial_count: u64,
    pub incorrect_count: u64,
    pub unscorable_count: u64,
    pub answer_rate: f64,
    pub skip_rate: f64,
    pub correct_rate: f64,
    pub average_answer_seconds: u64,
    pub coverage_gap_count: u64,
    pub coverage_gap_turn_count: u64,
    pub coverage_gap_domain_metrics: Vec<LearningQuestionCoverageGapDimensionMetric>,
    pub coverage_gap_reason_metrics: Vec<LearningQuestionCoverageGapDimensionMetric>,
    pub question_metrics: Vec<LearningProbeQuestionMetrics>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningQuestionCoverageGapDimensionMetric {
    pub code: String,
    pub count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeQuestionMetrics {
    pub question_id: String,
    pub question_version: u32,
    pub knowledge_code: String,
    pub question_text: String,
    pub jurisdiction_code: String,
    pub platform_code: String,
    pub business_stage_code: String,
    pub source_publishers: Vec<String>,
    pub source_valid_through: Option<String>,
    pub curriculum_review_required: bool,
    pub production_approved: bool,
    pub issued_count: u64,
    pub active_count: u64,
    pub answered_count: u64,
    pub skipped_count: u64,
    pub expired_count: u64,
    pub superseded_count: u64,
    pub correct_count: u64,
    pub partial_count: u64,
    pub incorrect_count: u64,
    pub unscorable_count: u64,
    pub answer_rate: f64,
    pub skip_rate: f64,
    pub correct_rate: f64,
    pub average_answer_seconds: u64,
    pub choice_metrics: Vec<LearningProbeChoiceMetrics>,
    pub unmapped_choice_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningProbeChoiceMetrics {
    pub choice_id: String,
    pub choice_text: String,
    pub selected_count: u64,
    pub selected_rate: f64,
}
