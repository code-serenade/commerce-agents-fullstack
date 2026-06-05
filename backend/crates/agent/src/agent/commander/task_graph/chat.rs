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
            skill_autosuggest: Some(role != "chat"),
            timeout_sec: None,
            input_refs: Vec::new(),
            acceptance: vec![DEFAULT_NODE_ACCEPTANCE.to_string()],
            max_attempts: 1,
        }],
        adaptive_hint: None,
    }
}

pub(super) fn plan_selected_roles_graph(
    task_id: &TaskId,
    task: &str,
    roles: &[String],
) -> TaskGraph {
    let roles = roles
        .iter()
        .map(|role| role.trim())
        .filter(|role| !role.is_empty() && *role != "chat")
        .fold(Vec::<String>::new(), |mut unique, role| {
            if !unique.iter().any(|item| item == role) {
                unique.push(role.to_string());
            }
            unique
        });
    if roles.len() <= 1 {
        return plan_single_chat_graph(
            task_id,
            task,
            roles.first().map(String::as_str).unwrap_or("chat"),
        );
    }

    let has_search = roles.iter().any(|role| role == "search");
    let mut nodes = roles
        .iter()
        .map(|role| {
            let depends_on = if has_search && role != "search" {
                vec!["search".to_string()]
            } else {
                Vec::new()
            };
            selected_role_node(role, task, depends_on, true)
        })
        .collect::<Vec<_>>();
    let primary_role = selected_roles_final_role(&roles);
    nodes.push(selected_role_node(
        &format!("{primary_role}_final"),
        &format!(
            "把本轮用户手动指定的岗位输入合成一份自然可读的最终回答：{task}。\
             只能整合实际执行的已选岗位，不补写未选择岗位，不展示 worker \
             report、任务图、role_output、context 字段、岗位失败诊断或变量名。"
        ),
        roles.clone(),
        false,
    ));

    TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes,
        adaptive_hint: None,
    }
}

fn selected_roles_final_role(roles: &[String]) -> String {
    roles
        .iter()
        .find(|role| role.as_str() == "ops")
        .or_else(|| roles.iter().find(|role| role.as_str() != "search"))
        .cloned()
        .or_else(|| roles.first().cloned())
        .unwrap_or_else(|| "ops".to_string())
}

fn selected_role_node(
    id_or_role: &str,
    objective: &str,
    depends_on: Vec<String>,
    autosuggest: bool,
) -> TaskNode {
    let role = id_or_role.strip_suffix("_final").unwrap_or(id_or_role);
    TaskNode {
        id: TaskNodeId(id_or_role.to_string()),
        role: role.to_string(),
        worker_id: WorkerId(format!("worker.{role}")),
        title: role.to_string(),
        objective: objective.to_string(),
        llm_usage: Some("fast".to_string()),
        skill_autosuggest: Some(autosuggest),
        timeout_sec: None,
        input_refs: depends_on.into_iter().map(TaskNodeId).collect(),
        acceptance: vec![DEFAULT_NODE_ACCEPTANCE.to_string()],
        max_attempts: 1,
    }
}
