use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    common::{metadata_csv_list, metadata_non_empty, metadata_string_value, metadata_u64_value},
    specs::coordination_status_query_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_string_param,
};

pub struct CoordinationStatusQuery {
    spec: SkillSpec,
}

impl CoordinationStatusQuery {
    pub fn new() -> Self {
        Self {
            spec: coordination_status_query_spec(),
        }
    }
}

impl Default for CoordinationStatusQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationStatusQuery {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let query_type = normalize_query_type(
            optional_string_param(&params, "query_type")?
                .unwrap_or_else(|| infer_status_query_type(&context.metadata).to_owned()),
        );
        let visible_scope = visible_scope(&context);
        let session = session_status_summary(&context);
        let worker = worker_status_summary(&context);
        let task = task_status_summary(&context);
        let workspace = workspace_status_summary(&context);
        let context_status = context_readiness_summary(&context);
        let health = system_health_summary(&context);
        let blockers = coordination_blockers(&context);
        let observability = observability_limits(&context);
        let progress_view = progress_view_summary(&worker, &task, &workspace, &context_status);
        let role_statuses = role_statuses_summary(&worker, &task, &session);
        let next_actions = status_next_actions(
            &query_type,
            &worker,
            &task,
            &workspace,
            &context_status,
            &blockers,
        );
        let user_visible_summary =
            user_visible_status_summary(&query_type, &worker, &task, &workspace, &progress_view);

        let summary = status_summary_line(&query_type, &worker, &task, &workspace, &health);
        Ok(SkillOutcome::new(json!({
            "status": "success",
            "query_type": query_type,
            "scope": visible_scope,
            "session": session,
            "worker": worker,
            "task": task,
            "workspace": workspace,
            "context": context_status,
            "system_health": health,
            "blockers": blockers,
            "observability": observability,
            "progress_view": progress_view,
            "role_statuses": role_statuses,
            "next_actions": next_actions,
            "user_visible_summary": user_visible_summary,
        }))
        .with_summary(summary)
        .with_metadata("status_query_scope", json!("session_local"))
        .with_metadata("status_query_type", json!(query_type)))
    }
}

fn normalize_query_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "agent" | "agent_status" => "agent_status".to_string(),
        "task" | "task_status" => "task_status".to_string(),
        "workspace" | "workspace_status" => "workspace_status".to_string(),
        "session" | "session_status" => "session_status".to_string(),
        "health" | "system" | "system_health" => "system_health".to_string(),
        _ => "system_health".to_string(),
    }
}

fn infer_status_query_type(metadata: &Map<String, Value>) -> &'static str {
    if metadata.contains_key("workspace_id") || metadata.contains_key("message.workspace_id") {
        "workspace_status"
    } else if metadata.contains_key("graph_id")
        || metadata.contains_key("node_id")
        || metadata.contains_key("task_id")
    {
        "task_status"
    } else if metadata.contains_key("session_id") || metadata.contains_key("message.session_id") {
        "session_status"
    } else {
        "system_health"
    }
}

fn visible_scope(context: &SkillContext) -> Value {
    json!({
        "session_id": context.session_id.clone(),
        "agent_id": context.agent_id.clone(),
        "workspace_id": metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id")),
        "run_id": metadata_string_value(&context.metadata, "request.workspace.run_id"),
        "conversation_id": metadata_string_value(&context.metadata, "request.context.conversation_id"),
        "user_id": metadata_string_value(&context.metadata, "message.user_id"),
        "visibility": "current_session_only",
    })
}

fn session_status_summary(context: &SkillContext) -> Value {
    let selected_roles = metadata_csv_list(&context.metadata, "request.strategy.selected_roles");
    let support_roles = metadata_csv_list(
        &context.metadata,
        "request.role_route.support_runtime_roles",
    );
    let refs = metadata_csv_list(&context.metadata, "request.context.refs");
    json!({
        "session_id": context.session_id.clone(),
        "current_agent_id": context.agent_id.clone(),
        "selected_roles": selected_roles,
        "primary_role": metadata_string_value(&context.metadata, "request.strategy.primary_role")
            .or_else(|| metadata_string_value(&context.metadata, "request.role_route.primary_runtime_role"))
            .or_else(|| metadata_string_value(&context.metadata, "role_runtime")),
        "support_roles": support_roles,
        "response_mode": metadata_string_value(&context.metadata, "request.strategy.response_mode"),
        "collaboration_mode": metadata_string_value(&context.metadata, "request.strategy.collaboration_mode"),
        "context_ref_count": refs.len(),
    })
}

fn worker_status_summary(context: &SkillContext) -> Value {
    let worker_role = metadata_string_value(&context.metadata, "role_runtime");
    let request_reason = metadata_string_value(&context.metadata, "request_reason");
    let attempt = metadata_u64_value(&context.metadata, "attempt").unwrap_or(1);
    json!({
        "worker_agent_id": context.agent_id.clone(),
        "runtime_role": worker_role,
        "status": if request_reason.is_some() { "executing_skill" } else { "ready" },
        "attempt": attempt,
        "request_reason": request_reason,
    })
}

fn task_status_summary(context: &SkillContext) -> Value {
    let upstream_roles = metadata_csv_list(&context.metadata, "assignment_input_roles");
    let input_count = metadata_u64_value(&context.metadata, "assignment_input_count").unwrap_or(0);
    let readiness = if input_count == 0 {
        "independent"
    } else if upstream_roles.is_empty() {
        "waiting_upstream_inputs"
    } else {
        "ready_with_upstream_inputs"
    };
    json!({
        "task_id": metadata_string_value(&context.metadata, "task_id"),
        "task_description": metadata_string_value(&context.metadata, "task_description"),
        "graph_id": metadata_string_value(&context.metadata, "graph_id"),
        "node_id": metadata_string_value(&context.metadata, "node_id"),
        "assignment_role": metadata_string_value(&context.metadata, "assignment_role"),
        "attempt": metadata_u64_value(&context.metadata, "attempt").unwrap_or(1),
        "input_count": input_count,
        "upstream_roles": upstream_roles,
        "readiness": readiness,
        "rework_instruction": metadata_string_value(&context.metadata, "assignment_rework_instruction"),
    })
}

fn workspace_status_summary(context: &SkillContext) -> Value {
    json!({
        "workspace_id": metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id")),
        "run_id": metadata_string_value(&context.metadata, "request.workspace.run_id"),
        "conversation_id": metadata_string_value(&context.metadata, "request.context.conversation_id"),
        "workspace_context_readiness": metadata_string_value(&context.metadata, "request.workspaceContext.readiness"),
        "workspace_context_pack_readiness": metadata_string_value(&context.metadata, "request.workspaceContextPack.readiness"),
    })
}

fn context_readiness_summary(context: &SkillContext) -> Value {
    json!({
        "product_context_loaded": metadata_non_empty(&context.metadata, "request.context.selected_summary"),
        "workspace_context_loaded": metadata_non_empty(&context.metadata, "request.workspaceContext.summary"),
        "workspace_context_pack_loaded": metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary"),
        "product_context_refs": metadata_csv_list(&context.metadata, "request.context.refs"),
        "workspace_context_refs": metadata_csv_list(&context.metadata, "request.workspaceContext.refs"),
        "workspace_pack_refs": metadata_csv_list(&context.metadata, "request.workspaceContextPack.refs"),
    })
}

fn system_health_summary(context: &SkillContext) -> Value {
    let has_session = context.session_id.is_some();
    let has_agent = context.agent_id.is_some();
    let has_task = metadata_non_empty(&context.metadata, "task_id");
    let has_context = metadata_non_empty(&context.metadata, "request.context.selected_summary")
        || metadata_non_empty(&context.metadata, "request.workspaceContext.summary")
        || metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary");
    let health = if has_session && has_agent && has_task {
        "active"
    } else if has_session || has_agent {
        "partial"
    } else {
        "limited_visibility"
    };
    json!({
        "health": health,
        "has_session": has_session,
        "has_agent": has_agent,
        "has_task": has_task,
        "has_context": has_context,
        "health_basis": "skill-context-visible-runtime-state",
    })
}

fn coordination_blockers(context: &SkillContext) -> Vec<String> {
    let mut blockers = Vec::new();
    if context.session_id.is_none() {
        blockers.push("当前 skill 调用没有 session_id，无法判断完整会话进度。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "task_id") {
        blockers.push("当前上下文没有 task_id，无法判断具体任务节点状态。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "request.workspace.run_id") {
        blockers.push("当前上下文没有 run_id，无法关联完整工作区运行轨迹。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "request.workspaceContext.summary")
        && !metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary")
    {
        blockers.push("当前工作区上下文尚未完整加载，可见来源有限。".to_string());
    }
    blockers
}

fn observability_limits(context: &SkillContext) -> Value {
    json!({
        "global_cluster_visibility": false,
        "database_live_query": false,
        "platform_adapter_probe": false,
        "other_sessions_visible": false,
        "current_session_scope": true,
        "metadata_keys": context.metadata.keys().cloned().collect::<Vec<_>>(),
    })
}

fn progress_view_summary(
    worker: &Value,
    task: &Value,
    workspace: &Value,
    context_status: &Value,
) -> Value {
    let input_count = task.get("input_count").and_then(Value::as_u64).unwrap_or(0);
    let has_task = task.get("task_id").and_then(Value::as_str).is_some();
    let has_workspace_run = workspace.get("run_id").and_then(Value::as_str).is_some();
    let workspace_loaded = context_status
        .get("workspace_context_loaded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || context_status
            .get("workspace_context_pack_loaded")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let product_loaded = context_status
        .get("product_context_loaded")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let worker_status = worker
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("ready");

    let (phase, percent, label) = if !has_task {
        (
            "limited_visibility",
            10,
            "当前 skill 只能看到会话级上下文，未收到具体任务节点信息。",
        )
    } else if worker_status == "executing_skill" && input_count > 0 {
        (
            "running_with_upstream_context",
            67,
            "当前岗位已收到上游岗位输入，正在执行状态查询或补充本岗位结果。",
        )
    } else if worker_status == "executing_skill" {
        (
            "running_independent_task",
            50,
            "当前岗位正在执行本轮状态查询或独立任务。",
        )
    } else if input_count > 0 {
        (
            "ready_with_upstream_context",
            55,
            "当前岗位已具备上游输入，可继续产出本岗位结果。",
        )
    } else {
        ("ready", 35, "当前岗位已收到任务，尚未看到上游岗位输入。")
    };

    let mut basis = Vec::new();
    if has_task {
        basis.push("task_context");
    }
    if input_count > 0 {
        basis.push("upstream_worker_inputs");
    }
    if has_workspace_run {
        basis.push("workspace_run");
    }
    if workspace_loaded {
        basis.push("workspace_context");
    }
    if product_loaded {
        basis.push("selected_product_context");
    }
    if basis.is_empty() {
        basis.push("skill_context");
    }

    json!({
        "phase": phase,
        "percent": percent,
        "label": label,
        "basis": basis,
        "workspace_bound": has_workspace_run,
        "upstream_input_count": input_count,
    })
}

fn role_statuses_summary(worker: &Value, task: &Value, session: &Value) -> Value {
    let current_role = worker
        .get("runtime_role")
        .and_then(Value::as_str)
        .or_else(|| task.get("assignment_role").and_then(Value::as_str));
    let upstream_roles = task
        .get("upstream_roles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let selected_roles = session
        .get("selected_roles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let support_roles = session
        .get("support_roles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut statuses = Vec::new();
    for role in upstream_roles.iter().filter_map(Value::as_str) {
        push_role_status(
            &mut statuses,
            role,
            "upstream_completed",
            "upstream",
            "assignment_input_roles",
        );
    }
    if let Some(role) = current_role {
        push_role_status(&mut statuses, role, "working", "current", "role_runtime");
    }
    for role in support_roles.iter().filter_map(Value::as_str) {
        push_role_status(
            &mut statuses,
            role,
            "planned_or_support",
            "support",
            "request.role_route.support_runtime_roles",
        );
    }
    for role in selected_roles.iter().filter_map(Value::as_str) {
        push_role_status(
            &mut statuses,
            role,
            "selected",
            "selected",
            "request.strategy.selected_roles",
        );
    }

    json!(statuses)
}

fn push_role_status(
    statuses: &mut Vec<Value>,
    role: &str,
    state: &str,
    relation: &str,
    source: &str,
) {
    if role.trim().is_empty() {
        return;
    }
    if statuses.iter().any(|item| {
        item.get("role")
            .and_then(Value::as_str)
            .is_some_and(|existing| existing == role)
    }) {
        return;
    }
    statuses.push(json!({
        "role": role,
        "state": state,
        "relation": relation,
        "source": source,
    }));
}

fn status_next_actions(
    query_type: &str,
    worker: &Value,
    task: &Value,
    workspace: &Value,
    context_status: &Value,
    blockers: &[String],
) -> Vec<String> {
    let mut actions = Vec::new();
    let has_task = task.get("task_id").and_then(Value::as_str).is_some();
    let input_count = task.get("input_count").and_then(Value::as_u64).unwrap_or(0);
    let worker_status = worker
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("ready");
    let has_workspace_run = workspace.get("run_id").and_then(Value::as_str).is_some();
    let workspace_loaded = context_status
        .get("workspace_context_loaded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || context_status
            .get("workspace_context_pack_loaded")
            .and_then(Value::as_bool)
            .unwrap_or(false);

    match query_type {
        "workspace_status" if !has_workspace_run => {
            actions.push("本轮未绑定 workspace run，只能展示会话级状态。".to_string());
        }
        "task_status" if has_task && input_count > 0 => {
            actions.push("继续读取上游岗位结果，并等待当前岗位完成本轮输出。".to_string());
        }
        "agent_status" if worker_status == "executing_skill" => {
            actions
                .push("当前岗位正在执行 skill，等待 skill_result 回写后再汇总结论。".to_string());
        }
        "session_status" => {
            actions.push("保持在当前会话内追问；跨会话历史不会自动混入本轮状态。".to_string());
        }
        _ => {
            actions.push("继续等待当前 Agent runtime 通过 SSE 推送后续事件。".to_string());
        }
    }

    if !workspace_loaded {
        actions.push("如需引用历史资料或交付物，请在本轮显式选择对应来源。".to_string());
    }
    if blockers.is_empty() {
        actions.push("当前可见范围内没有发现阻塞项。".to_string());
    } else {
        actions.push("先处理 blockers 中列出的可见性或上下文缺口。".to_string());
    }
    actions
}

fn user_visible_status_summary(
    query_type: &str,
    worker: &Value,
    task: &Value,
    workspace: &Value,
    progress_view: &Value,
) -> String {
    let role = worker
        .get("runtime_role")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let task_id = task
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let workspace_id = workspace
        .get("workspace_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let phase = progress_view
        .get("phase")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let percent = progress_view
        .get("percent")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    format!(
        "当前查询 {query_type}：{role} 正在处理 {task_id}，工作区 {workspace_id}，阶段 \
         {phase}，可见进度约 {percent}%。"
    )
}

fn status_summary_line(
    query_type: &str,
    worker: &Value,
    task: &Value,
    workspace: &Value,
    health: &Value,
) -> String {
    let role = worker
        .get("runtime_role")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let task_id = task
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let workspace_id = workspace
        .get("workspace_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let health_name = health
        .get("health")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    format!(
        "状态查询已完成：query_type={query_type}，role={role}，task={task_id}，\
         workspace={workspace_id}，health={health_name}"
    )
}
