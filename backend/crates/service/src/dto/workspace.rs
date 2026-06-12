use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::dto::campaign::CampaignDraftResponse;

#[derive(Debug, Clone, Deserialize)]
pub struct ListWorkspacesRequest {
    pub user_id: String,
    pub status: Option<String>,
    pub product_id: Option<String>,
    pub campaign_id: Option<String>,
    pub workspace_type: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub user_id: String,
    pub product_id: Option<String>,
    pub campaign_id: Option<String>,
    pub title: String,
    pub workspace_type: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceRequest {
    pub user_id: String,
    pub workspace_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub product_id: Option<String>,
    pub campaign_id: Option<String>,
    pub title: Option<String>,
    pub workspace_type: Option<String>,
    pub status: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWorkspaceRequest {
    pub user_id: String,
    pub workspace_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListWorkspaceTasksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub status: Option<String>,
    pub owner_role: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub depends_on: Option<Value>,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub depends_on: Option<Value>,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWorkspaceTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListWorkspaceRunsRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub status: Option<String>,
    pub source: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub goal: String,
    pub requirements: Option<Value>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub summary: Option<String>,
    pub report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub goal: Option<String>,
    pub requirements: Option<Value>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub summary: Option<String>,
    pub report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListWorkspaceRunTasksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRunTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub workspace_task_id: Option<String>,
    pub seq: Option<i32>,
    pub owner_role: Option<String>,
    pub status: Option<String>,
    pub attempts: Option<i32>,
    pub acceptance_score: Option<f64>,
    pub acceptance_passed: Option<bool>,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceRunTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub run_task_id: String,
    pub seq: Option<i32>,
    pub owner_role: Option<String>,
    pub status: Option<String>,
    pub attempts: Option<i32>,
    pub acceptance_score: Option<f64>,
    pub acceptance_passed: Option<bool>,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListWorkspaceArtifactsRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub run_task_id: Option<String>,
    pub artifact_type: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub run_task_id: Option<String>,
    pub artifact_type: String,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: Option<String>,
    pub storage_kind: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub source: Option<String>,
    pub validation_status: Option<String>,
    pub validation_report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub artifact_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspaceArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub artifact_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: Option<String>,
    pub storage_kind: Option<String>,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub status: Option<String>,
    pub validation_status: Option<String>,
    pub validation_report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWorkspaceArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub artifact_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportWorkspaceArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub artifact_id: String,
    pub format: Option<String>,
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceContextSummaryRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub include_workspace_product_context: bool,
    pub include_implicit_conversation_files: bool,
    pub run_id: Option<String>,
    pub context_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWorkspaceContextPackRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub include_workspace_product_context: bool,
    pub include_implicit_conversation_files: bool,
    pub run_id: Option<String>,
    pub context_refs: Vec<String>,
    pub max_items: Option<usize>,
    pub max_chars: Option<usize>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnsureAgentWorkspaceRequest {
    pub user_id: String,
    pub requested_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListExecutionQueueRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub status: Option<String>,
    pub action_type: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExecutionQueueRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub action_type: String,
    pub description: String,
    pub platform: Option<String>,
    pub payload: Option<Value>,
    pub expected_impact: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExecutionQueueRequest {
    pub user_id: String,
    pub execution_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecideExecutionQueueRequest {
    pub user_id: String,
    pub execution_id: String,
    pub reason: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkAgentWorkspaceRunRequest {
    pub user_id: String,
    pub requested_workspace_id: Option<String>,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub input: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentWorkspaceTaskNode {
    pub graph_id: String,
    pub node_id: String,
    pub worker_id: String,
    pub role: String,
    pub title: String,
    pub objective: String,
    pub input_refs: Vec<String>,
    pub acceptance: Vec<String>,
    pub max_attempts: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkAgentWorkspaceTasksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub session_id: String,
    pub root_task: String,
    pub nodes: Vec<AgentWorkspaceTaskNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkAgentWorkspaceTaskStartedRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub graph_id: String,
    pub node_id: String,
    pub worker_id: String,
    pub attempt: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAgentWorkspaceTaskReportRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub graph_id: String,
    pub node_id: String,
    pub task_id: String,
    pub worker_id: String,
    pub role: String,
    pub content: String,
    pub role_output: Option<Value>,
    pub evidence: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAgentWorkspaceTaskEvaluationRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub graph_id: String,
    pub node_id: String,
    pub passed: bool,
    pub score: f64,
    pub reasons: Vec<String>,
    pub rework_instruction: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAgentSearchEvidenceRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub status: String,
    pub summary: Option<String>,
    pub execution_mode: Option<String>,
    pub output: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentSkillRunsRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub status: Option<String>,
    pub skill_id: Option<String>,
    pub role: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAgentSkillRequestedRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub session_id: Option<String>,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAgentSkillResultRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub session_id: Option<String>,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub status: String,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub output: Option<Value>,
    pub execution_mode: Option<String>,
    pub risk_level: Option<String>,
    pub approval_required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterializeAgentWorkspaceTaskRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub output: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterializeAgentWorkspaceMemoryRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub output: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterializeAgentCampaignDraftRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub output: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterializeAgentContentArtifactRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub output: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaterializedAgentCampaignDraftResponse {
    pub campaign: CampaignDraftResponse,
    pub artifact: Option<WorkspaceArtifactResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompleteAgentWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteAgentWorkspaceRunResponse {
    pub run: WorkspaceRunResponse,
    pub artifact: Option<WorkspaceArtifactResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateAgentWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub artifact_id: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentQualityChecksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitAgentRunFeedbackRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentLearningRecordsRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: Option<String>,
    pub role: Option<String>,
    pub category: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentTrustScoresRequest {
    pub user_id: String,
    pub role: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildAgentLearningContextRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub role: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentQualityCheckListResponse {
    pub checks: Vec<AgentQualityCheckResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentLearningRecordListResponse {
    pub records: Vec<AgentLearningRecordResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTrustScoreListResponse {
    pub scores: Vec<AgentTrustScoreResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkillRunListResponse {
    pub skill_runs: Vec<AgentSkillRunResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub agent_context: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkillRunResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub session_id: Option<String>,
    pub worker_id: String,
    pub task_id: String,
    pub request_id: String,
    pub skill_id: String,
    pub role: String,
    pub status: String,
    pub reason: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub input: Value,
    pub output: Value,
    pub execution_mode: Option<String>,
    pub risk_level: Option<String>,
    pub approval_required: bool,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub requested_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentQualityEvaluationResponse {
    pub quality_check: AgentQualityCheckResponse,
    pub trust_update: AgentTrustUpdateResponse,
    pub learning_records: Vec<AgentLearningRecordResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRunFeedbackResponse {
    pub learning_record: AgentLearningRecordResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentLearningContextResponse {
    pub trust_summary: Value,
    pub learning_summary: String,
    pub records: Vec<AgentLearningRecordResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentQualityCheckResponse {
    pub id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub artifact_id: Option<String>,
    pub user_id: String,
    pub role: String,
    pub score: f64,
    pub passed: bool,
    pub dimensions: Value,
    pub issues: Value,
    pub suggestions: Value,
    pub source: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTrustScoreResponse {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub skill_id: Option<String>,
    pub score: f64,
    pub trust_level: String,
    pub previous_score: Option<f64>,
    pub previous_level: Option<String>,
    pub delta: f64,
    pub reason: Option<String>,
    pub source: String,
    pub quality_check_id: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentTrustUpdateResponse {
    pub role: String,
    pub skill_id: Option<String>,
    pub previous_score: f64,
    pub new_score: f64,
    pub previous_level: String,
    pub new_level: String,
    pub delta: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentLearningRecordResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub quality_check_id: Option<String>,
    pub role: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub source_action: String,
    pub feedback_rating: Option<i32>,
    pub feedback_comment: Option<String>,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FailAgentWorkspaceRunRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceTaskListResponse {
    pub tasks: Vec<WorkspaceTaskResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRunListResponse {
    pub runs: Vec<WorkspaceRunResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceArtifactListResponse {
    pub artifacts: Vec<WorkspaceArtifactResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceArtifactExportResponse {
    pub artifact_id: String,
    pub workspace_id: String,
    pub title: String,
    pub file_name: String,
    pub mime_type: String,
    pub format: String,
    pub content: String,
    pub content_chars: usize,
    pub source_format: String,
    pub source_storage_kind: String,
    pub metadata: Value,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextSummaryResponse {
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub run_id: Option<String>,
    pub readiness: String,
    pub summary: String,
    pub counts: WorkspaceContextSummaryCounts,
    pub attachment_sources: Vec<WorkspaceContextSourceResponse>,
    pub campaign_sources: Vec<WorkspaceContextSourceResponse>,
    pub product_asset_sources: Vec<WorkspaceContextSourceResponse>,
    pub product_material_sources: Vec<WorkspaceContextSourceResponse>,
    pub artifact_sources: Vec<WorkspaceContextSourceResponse>,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
    pub agent_context: Value,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextSummaryCounts {
    pub campaigns: usize,
    pub conversation_files: usize,
    pub product_assets: usize,
    pub product_materials: usize,
    pub workspace_artifacts: usize,
}

impl WorkspaceContextSummaryCounts {
    pub fn total(&self) -> usize {
        self.campaigns
            + self.conversation_files
            + self.product_assets
            + self.product_materials
            + self.workspace_artifacts
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextSourceResponse {
    pub source_type: String,
    pub id: String,
    pub title: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub status: String,
    pub ref_key: String,
    pub preview: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPackResponse {
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub run_id: Option<String>,
    pub readiness: String,
    pub summary: String,
    pub counts: WorkspaceContextSummaryCounts,
    pub items: Vec<WorkspaceContextPackItemResponse>,
    pub omitted: Vec<String>,
    pub omitted_details: Vec<WorkspaceContextPackOmittedResponse>,
    pub warnings: Vec<String>,
    pub agent_context: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPackItemResponse {
    pub source_type: String,
    pub id: String,
    pub title: Option<String>,
    pub ref_key: String,
    pub content_kind: String,
    pub text: Option<String>,
    pub text_chars: usize,
    pub truncated: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPackOmittedResponse {
    pub ref_key: String,
    pub reason: String,
    pub source_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionQueueListResponse {
    pub items: Vec<ExecutionQueueResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkedAgentWorkspaceRunResponse {
    pub workspace: WorkspaceResponse,
    pub run: WorkspaceRunResponse,
    pub created_workspace: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkedAgentWorkspaceTasksResponse {
    pub tasks: Vec<WorkspaceRunTaskResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: Option<String>,
    pub campaign_id: Option<String>,
    pub title: String,
    pub workspace_type: String,
    pub status: String,
    pub config: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceTaskResponse {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: String,
    pub priority: i32,
    pub depends_on: Value,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRunResponse {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub goal: String,
    pub requirements: Value,
    pub status: String,
    pub source: String,
    pub summary: Option<String>,
    pub report: Value,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRunTaskResponse {
    pub id: String,
    pub run_id: String,
    pub workspace_id: String,
    pub workspace_task_id: Option<String>,
    pub user_id: String,
    pub seq: i32,
    pub owner_role: Option<String>,
    pub status: String,
    pub attempts: i32,
    pub acceptance_score: f64,
    pub acceptance_passed: bool,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceArtifactResponse {
    pub id: String,
    pub workspace_id: String,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub run_task_id: Option<String>,
    pub user_id: String,
    pub artifact_type: String,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: String,
    pub storage_kind: String,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub source: String,
    pub status: String,
    pub validation_status: String,
    pub validation_report: Value,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionQueueResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub action_type: String,
    pub description: String,
    pub platform: Option<String>,
    pub payload: Value,
    pub expected_impact: Option<String>,
    pub status: String,
    pub decision_reason: Option<String>,
    pub result: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub decided_at: Option<OffsetDateTime>,
}
