use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

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
pub struct ExecutionQueueListResponse {
    pub items: Vec<ExecutionQueueResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
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
