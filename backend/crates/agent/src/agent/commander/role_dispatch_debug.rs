use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{OpenOptions, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tracing::warn;

use crate::{
    core::protocol::{
        MessageContext, TaskGraph, WorkerAssignment, WorkerProfile, WorkerReport,
        WorkerReportStatus,
    },
    roles::RoleRoute,
};

const INPUT_PREVIEW_CHARS: usize = 120;
const LOG_FILE_NAME: &str = "role-dispatch-debug.jsonl";

pub(crate) fn log_request_received(
    context: &MessageContext,
    input: &str,
    worker_profiles: &[WorkerProfile],
) {
    append_event(json!({
        "event": "request_received",
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "input_preview": preview(input),
        "registered_workers": registered_worker_summary(worker_profiles),
    }));
}

pub(crate) fn log_route_decided(
    context: &MessageContext,
    role_route: Option<&RoleRoute>,
    worker_profiles: &[WorkerProfile],
) {
    let route = role_route.map(|route| {
        json!({
            "primary_role": route.primary_role.0,
            "primary_runtime_role": route.primary_runtime_role,
            "support_roles": route
                .support_roles
                .iter()
                .map(|role| role.0.clone())
                .collect::<Vec<_>>(),
            "support_runtime_roles": route.support_runtime_roles,
            "scores": route
                .scores
                .iter()
                .map(|score| json!({
                    "role_id": score.role_id.0,
                    "runtime_role": score.runtime_role,
                    "score": score.score,
                    "keyword_hits": score.keyword_hits,
                }))
                .collect::<Vec<_>>(),
        })
    });
    append_event(json!({
        "event": "route_decided",
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "route": route,
        "registered_workers": registered_worker_summary(worker_profiles),
    }));
}

pub(crate) fn log_task_graph_planned(
    context: &MessageContext,
    route_mode: &str,
    graph: &TaskGraph,
    worker_profiles: &[WorkerProfile],
) {
    let registered_worker_ids = worker_profiles
        .iter()
        .map(|profile| profile.worker_id.0.as_str())
        .collect::<BTreeSet<_>>();
    append_event(json!({
        "event": "task_graph_planned",
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "route_mode": route_mode,
        "graph_id": graph.graph_id.0,
        "node_count": graph.nodes.len(),
        "nodes": graph
            .nodes
            .iter()
            .map(|node| json!({
                "node_id": node.id.0,
                "role": node.role,
                "worker_id": node.worker_id.0,
                "worker_registered": registered_worker_ids.contains(node.worker_id.0.as_str()),
                "input_refs": node
                    .input_refs
                    .iter()
                    .map(|item| item.0.clone())
                    .collect::<Vec<_>>(),
                "llm_usage": node.llm_usage,
                "skill_autosuggest": node.skill_autosuggest,
                "max_attempts": node.max_attempts,
            }))
            .collect::<Vec<_>>(),
        "registered_workers": registered_worker_summary(worker_profiles),
    }));
}

pub(crate) fn log_assignment_dispatch(
    context: &MessageContext,
    assignment: &WorkerAssignment,
    worker_profiles: &[WorkerProfile],
    dispatched: bool,
    reason: Option<&str>,
) {
    append_event(json!({
        "event": if dispatched { "assignment_dispatched" } else { "assignment_dispatch_failed" },
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "assignment": assignment_summary(assignment),
        "dispatched": dispatched,
        "failure_reason": reason,
        "registered_workers": registered_worker_summary(worker_profiles),
    }));
}

pub(crate) fn log_worker_report(context: &MessageContext, report: &WorkerReport) {
    append_event(json!({
        "event": "worker_report_received",
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "report": {
            "graph_id": report.graph_id.0,
            "node_id": report.node_id.0,
            "task_id": report.task_id.0,
            "worker_id": report.worker_id.0,
            "role": report.role,
            "status": worker_status(&report.status),
            "diagnostics": report
                .diagnostics
                .iter()
                .map(|diagnostic| json!({
                    "kind": diagnostic.kind,
                    "source": diagnostic.source,
                    "fields": diagnostic.fields,
                }))
                .collect::<Vec<_>>(),
            "evidence_markers": report
                .evidence
                .iter()
                .filter(|item| item.contains("failure_kind:"))
                .cloned()
                .collect::<Vec<_>>(),
        },
    }));
}

pub(crate) fn log_task_node_failed(
    context: &MessageContext,
    graph_id: &str,
    node_id: &str,
    reason: &str,
) {
    append_event(json!({
        "event": "task_node_failed",
        "scope": "role_dispatch",
        "context": context_summary(context),
        "strategy": strategy_summary(context),
        "graph_id": graph_id,
        "node_id": node_id,
        "reason": preview(reason),
    }));
}

fn append_event(mut event: Value) {
    if let Some(object) = event.as_object_mut() {
        object.insert("ts_unix_ms".to_string(), json!(timestamp_ms()));
        object.insert(
            "log_file".to_string(),
            json!(log_path().display().to_string()),
        );
    }

    let path = log_path();
    if let Some(parent) = path.parent()
        && let Err(err) = create_dir_all(parent)
    {
        warn!(path = %parent.display(), error = %err, "failed to create role dispatch debug log dir");
        return;
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        warn!(path = %path.display(), "failed to open role dispatch debug log file");
        return;
    };
    match serde_json::to_string(&event) {
        Ok(line) => {
            if let Err(err) = writeln!(file, "{line}") {
                warn!(path = %path.display(), error = %err, "failed to write role dispatch debug log");
            }
        }
        Err(err) => {
            warn!(path = %path.display(), error = %err, "failed to serialize role dispatch debug log");
        }
    }
}

fn log_path() -> PathBuf {
    backend_logs_dir().join(LOG_FILE_NAME)
}

fn backend_logs_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.join("../.."))
        .join("logs")
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn context_summary(context: &MessageContext) -> Value {
    json!({
        "user_id": context.user_id.as_ref().map(|id| id.0.as_str()),
        "workspace_id": context.workspace_id.as_ref().map(|id| id.0.as_str()),
        "agent_id": context.agent_id.as_ref().map(|id| id.0.as_str()),
        "session_id": context.session_id.as_ref().map(|id| id.0.as_str()),
        "task_id": context.task_id.as_ref().map(|id| id.0.as_str()),
        "workspace_run_id": context.metadata.get("workspace.run_id"),
    })
}

fn strategy_summary(context: &MessageContext) -> Value {
    json!({
        "strategy_id": context.metadata.get("strategy.id"),
        "response_mode": context.metadata.get("strategy.response_mode"),
        "routing_mode": context.metadata.get("strategy.routing_mode"),
        "collaboration_mode": context.metadata.get("strategy.collaboration_mode"),
        "learning_level": context.metadata.get("strategy.learning_level"),
        "primary_role": context.metadata.get("strategy.primary_role"),
        "selected_roles": csv_list(context.metadata.get("strategy.selected_roles")),
        "workspace_context_readiness": context.metadata.get("workspaceContextPack.readiness"),
        "workspace_context_refs": csv_list(context.metadata.get("workspaceContextPack.refs")),
        "attachment_image_count": context.metadata.get("attachments.image_count"),
    })
}

fn registered_worker_summary(worker_profiles: &[WorkerProfile]) -> Value {
    json!({
        "count": worker_profiles.len(),
        "worker_ids": worker_profiles
            .iter()
            .map(|profile| profile.worker_id.0.clone())
            .collect::<Vec<_>>(),
        "runtime_roles": worker_profiles
            .iter()
            .filter_map(|profile| profile.worker_id.0.strip_prefix("worker.").map(str::to_string))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        "profiles": worker_profiles
            .iter()
            .map(|profile| json!({
                "worker_id": profile.worker_id.0,
                "agent_id": profile.agent_id,
                "name": profile.name,
                "status": profile.status,
                "capabilities": profile.capabilities,
            }))
            .collect::<Vec<_>>(),
    })
}

fn assignment_summary(assignment: &WorkerAssignment) -> Value {
    json!({
        "graph_id": assignment.graph_id.0,
        "node_id": assignment.node_id.0,
        "task_id": assignment.task_id.0,
        "worker_id": assignment.worker_id.0,
        "role": assignment.role,
        "attempt": assignment.attempt,
        "input_roles": assignment
            .inputs
            .iter()
            .map(|input| input.role.clone())
            .collect::<Vec<_>>(),
        "llm_usage": assignment.llm_usage,
        "skill_autosuggest": assignment.skill_autosuggest,
    })
}

fn csv_list(value: Option<&String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn preview(value: &str) -> String {
    let mut preview = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(INPUT_PREVIEW_CHARS)
        .collect::<String>();
    if value.chars().count() > INPUT_PREVIEW_CHARS {
        preview.push_str("...");
    }
    preview
}

fn worker_status(status: &WorkerReportStatus) -> &'static str {
    match status {
        WorkerReportStatus::Completed => "completed",
        WorkerReportStatus::Failed => "failed",
        WorkerReportStatus::TimedOut => "timed_out",
    }
}

#[cfg(test)]
mod tests {
    use super::preview;

    #[test]
    fn preview_truncates_long_input() {
        let text = "请分析这个商品的主图和卖点。".repeat(20);

        let preview = preview(&text);

        assert!(preview.chars().count() <= 123);
        assert!(preview.ends_with("..."));
    }
}
