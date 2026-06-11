use crate::agent::worker::Task;

pub(in crate::agent::worker::runner) fn inference_text(task: &Task) -> String {
    let mut parts = vec![task.description.clone()];
    for key in [
        "context.summary",
        "context.source_titles",
        "rag.summary",
        "rag.source_titles",
    ] {
        if let Some(value) = task.context.metadata.get(key) {
            parts.push(value.clone());
        }
    }
    if let Some(assignment) = &task.assignment {
        parts.push(assignment.objective.clone());
        parts.extend(assignment.inputs.iter().map(|input| input.content.clone()));
        if let Some(instruction) = &assignment.rework_instruction {
            parts.push(instruction.clone());
        }
    }
    parts.join("\n")
}

pub(in crate::agent::worker::runner) fn autosuggest_match_text(task: &Task) -> String {
    let mut parts = vec![task_user_description(task)];
    for key in [
        "context.summary",
        "context.source_titles",
        "rag.summary",
        "rag.source_titles",
        "context.selected_summary",
        "commerce.selected_product_context",
        "workspaceContext.summary",
        "workspaceContext.refs",
        "workspaceContextPack.summary",
        "workspaceContextPack.refs",
        "workspaceContextPack.contentRefs",
        "workspaceContextPack.readiness",
        "workspaceContextPack.items_json",
        "attachments.image_count",
        "attachments.image_refs",
        "attachments.images_json",
        "attachments.image_summaries_json",
        "attachments.ocr_text",
    ] {
        if let Some(value) = task.context.metadata.get(key).map(String::as_str) {
            push_metadata_part(&mut parts, key, value);
        }
    }

    if has_positive_count(task, "attachments.image_count")
        || metadata_non_empty(task, "attachments.image_refs")
        || metadata_non_empty(task, "attachments.images_json")
        || metadata_non_empty(task, "attachments.image_summaries_json")
    {
        parts.push("上传的图片".to_string());
    }
    if has_positive_count(task, "workspaceContextPack.item_count")
        || metadata_non_empty(task, "workspaceContextPack.refs")
        || metadata_non_empty(task, "workspaceContextPack.summary")
    {
        parts.push("上传文件 附件资料".to_string());
    }

    parts.join("\n")
}

pub(in crate::agent::worker::runner) fn task_user_description(task: &Task) -> String {
    let raw = task
        .assignment
        .as_ref()
        .map(|assignment| assignment.root_task.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| task.description.trim());

    current_user_request_text(raw)
}

pub(super) fn compact_task_text(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_CHARS: usize = 80;
    if trimmed.chars().count() <= MAX_CHARS {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_CHARS).collect::<String>();
    output.push_str("...");
    output
}

fn current_user_request_text(value: &str) -> String {
    let text = value.trim();
    for marker in [
        "当前用户请求：",
        "当前用户请求:",
        "Current user request:",
        "current user request:",
    ] {
        if let Some(index) = text.rfind(marker) {
            let request = text[index + marker.len()..].trim();
            if !request.is_empty() {
                return request.to_string();
            }
        }
    }
    text.to_string()
}

fn push_metadata_part(parts: &mut Vec<String>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        parts.push(format!("{key}: {value}"));
    }
}

fn metadata_non_empty(task: &Task, key: &str) -> bool {
    task.context
        .metadata
        .get(key)
        .is_some_and(|value| !value.trim().is_empty())
}

fn has_positive_count(task: &Task, key: &str) -> bool {
    task.context
        .metadata
        .get(key)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .is_some_and(|value| value > 0)
}

#[cfg(test)]
mod tests {
    use super::{autosuggest_match_text, task_user_description};
    use crate::{
        agent::worker::Task,
        core::protocol::{
            AgentId, MessageContext, TaskGraphId, TaskId, TaskNodeId, WorkerAssignment, WorkerId,
        },
    };

    fn assignment_task(root_task: &str, objective: &str) -> Task {
        Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: format!("{root_task}\n\n当前岗位任务：\n{objective}"),
            requester: AgentId("commander".to_string()),
            assignment: Some(WorkerAssignment {
                graph_id: TaskGraphId("graph-1".to_string()),
                node_id: TaskNodeId("data".to_string()),
                task_id: TaskId("task-1".to_string()),
                worker_id: WorkerId("worker.data".to_string()),
                role: "data".to_string(),
                attempt: 1,
                root_task: root_task.to_string(),
                objective: objective.to_string(),
                llm_usage: None,
                skill_autosuggest: None,
                timeout_sec: None,
                inputs: Vec::new(),
                rework_instruction: None,
            }),
        }
    }

    #[test]
    fn user_description_uses_current_request_from_history_wrapper() {
        let task = assignment_task(
            "同一会话历史上下文：\n上一轮让它做表格。\n\n当前用户请求：\n继续分析这个商品。",
            "没有用户上传的表格、Excel、CSV 或附件时，也要继续。",
        );

        assert_eq!(task_user_description(&task), "继续分析这个商品。");
        let text = autosuggest_match_text(&task);
        assert!(text.contains("继续分析这个商品。"));
        assert!(!text.contains("上一轮让它做表格"));
        assert!(!text.contains("没有用户上传的表格"));
    }
}
