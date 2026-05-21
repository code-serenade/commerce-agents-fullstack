use super::DEFAULT_NODE_ACCEPTANCE;
use crate::core::protocol::{TaskGraph, TaskGraphId, TaskId, TaskNode, TaskNodeId, WorkerId};

pub(super) fn plan_single_chat_graph(task_id: &TaskId, task: &str, role: &str) -> TaskGraph {
    let role = if role.trim().is_empty() { "chat" } else { role };
    TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes: vec![TaskNode {
            id: TaskNodeId(role.to_string()),
            role: role.to_string(),
            worker_id: WorkerId(format!("worker.{role}")),
            title: role.to_string(),
            objective: task.to_string(),
            llm_usage: Some("fast".to_string()),
            skill_autosuggest: Some(false),
            input_refs: Vec::new(),
            acceptance: vec![DEFAULT_NODE_ACCEPTANCE.to_string()],
            max_attempts: 1,
        }],
    }
}
