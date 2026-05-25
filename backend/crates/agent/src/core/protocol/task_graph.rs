use serde::{Deserialize, Serialize};

use crate::protocol::{TaskId, WorkerId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskGraphId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskNodeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraph {
    pub graph_id: TaskGraphId,
    pub root_task: String,
    pub nodes: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: TaskNodeId,
    pub role: String,
    pub worker_id: WorkerId,
    pub title: String,
    pub objective: String,
    #[serde(default)]
    pub llm_usage: Option<String>,
    #[serde(default)]
    pub skill_autosuggest: Option<bool>,
    pub input_refs: Vec<TaskNodeId>,
    pub acceptance: Vec<String>,
    pub max_attempts: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerAssignment {
    pub graph_id: TaskGraphId,
    pub node_id: TaskNodeId,
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub role: String,
    pub attempt: u8,
    #[serde(default)]
    pub root_task: String,
    pub objective: String,
    #[serde(default)]
    pub llm_usage: Option<String>,
    #[serde(default)]
    pub skill_autosuggest: Option<bool>,
    pub inputs: Vec<WorkerReport>,
    pub rework_instruction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerReport {
    pub graph_id: TaskGraphId,
    pub node_id: TaskNodeId,
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub role: String,
    pub content: String,
    pub role_output: Option<RoleWorkOutput>,
    pub evidence: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub status: WorkerReportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoleWorkOutput {
    pub summary: String,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub evidence: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerReportStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    pub node_id: TaskNodeId,
    pub passed: bool,
    pub score: f32,
    pub reasons: Vec<String>,
    pub rework_instruction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGraphProcessSummary {
    pub graph_id: TaskGraphId,
    pub root_task: String,
    pub primary: Option<TaskGraphProcessRole>,
    pub roles: Vec<TaskGraphProcessRole>,
    pub handoffs: Vec<TaskGraphHandoff>,
    pub next_actions: Vec<String>,
    pub check_items: Vec<String>,
    pub deliverables: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub incomplete: Vec<TaskGraphProcessGap>,
    pub panel: TaskGraphProcessPanel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGraphProcessRole {
    pub node_id: TaskNodeId,
    pub task_id: Option<TaskId>,
    pub worker_id: WorkerId,
    pub role: String,
    pub role_label: String,
    pub title: String,
    pub objective: String,
    pub status: String,
    pub attempt: u8,
    pub is_primary: bool,
    pub summary: Option<String>,
    pub finding: Option<String>,
    pub recommendation: Option<String>,
    pub evidence: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraphHandoff {
    pub from_node_id: TaskNodeId,
    pub to_node_id: TaskNodeId,
    pub from_role: String,
    pub to_role: String,
    pub from_role_label: String,
    pub to_role_label: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraphProcessGap {
    pub node_id: TaskNodeId,
    pub role: String,
    pub role_label: String,
    pub title: String,
    pub state: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraphProcessPanel {
    pub default_collapsed: bool,
    pub anchor: String,
    pub label: String,
    pub reason: String,
}
