use super::{
    types::ParsedSkillRequest,
    util::{compact_event_text, metadata_string, skill_status_name},
};
use crate::{
    agent::worker::Task,
    core::protocol::{AgentId, SessionEvent, TaskId, WorkerId},
    roles::{RoleProfile, RoleSkillCatalog},
    skills::{
        SkillContext, SkillExecutionResult, SkillExecutionStatus, SkillExecutor, SkillExecutorSet,
        SkillId, SkillRequest,
    },
};

pub(super) fn parse_skill_requests(output: &str) -> Vec<ParsedSkillRequest> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(output.trim()) else {
        return Vec::new();
    };
    value
        .get("skill_requests")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_skill_request)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(super) async fn execute_requested_skills(
    worker_id: &AgentId,
    role: &RoleProfile,
    task: &Task,
    output: &str,
    executor: Option<SkillExecutorSet>,
    event_tx: Option<&crate::agent_session::SessionEventTx>,
) -> Vec<SkillExecutionResult> {
    let requests = parse_skill_requests(output);
    if requests.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let Some(executor) = executor else {
        for (index, request) in requests.into_iter().enumerate() {
            let request_id = format!("{}-skill-{}", task.id, index + 1);
            emit_skill_requested(worker_id, role, task, &request_id, &request, event_tx);
            let result = skill_executor_unavailable_result(request_id, request.skill_id);
            emit_skill_result(worker_id, role, task, &result, event_tx);
            results.push(result);
        }
        return results;
    };

    for (index, request) in requests.into_iter().enumerate() {
        let request_id = format!("{}-skill-{}", task.id, index + 1);
        emit_skill_requested(worker_id, role, task, &request_id, &request, event_tx);
        if !RoleSkillCatalog::staged().can_role_use_skill(role, &request.skill_id) {
            let result = skill_not_allowed_result(request_id, request.skill_id, &role.runtime_role);
            emit_skill_result(worker_id, role, task, &result, event_tx);
            results.push(result);
            continue;
        }

        let context = build_skill_context(worker_id, role, task, &request, &request_id);
        let runtime_request =
            SkillRequest::new(request_id, request.skill_id, request.input).with_context(context);
        let result = executor.execute(runtime_request).await;
        emit_skill_result(worker_id, role, task, &result, event_tx);
        results.push(result);
    }

    results
}

fn parse_skill_request(value: &serde_json::Value) -> Option<ParsedSkillRequest> {
    let skill_id = value
        .get("skill_id")
        .or_else(|| value.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();
    let input = value
        .get("input")
        .or_else(|| value.get("arguments"))
        .or_else(|| value.get("args"))
        .or_else(|| value.get("parameters"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let reason = value
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    Some(ParsedSkillRequest {
        skill_id,
        input,
        reason,
    })
}

fn build_skill_context(
    worker_id: &AgentId,
    role: &RoleProfile,
    task: &Task,
    request: &ParsedSkillRequest,
    request_id: &str,
) -> SkillContext {
    let mut context = SkillContext::new()
        .with_invocation_id(request_id.to_string())
        .with_agent_id(worker_id.0.clone())
        .with_metadata("task_id", serde_json::json!(task.id))
        .with_metadata("task_description", serde_json::json!(task.description))
        .with_metadata("role_id", serde_json::json!(role.id.0))
        .with_metadata("role_name", serde_json::json!(role.name))
        .with_metadata("role_runtime", serde_json::json!(role.runtime_role));
    if let Some(session_id) = &task.context.session_id {
        context = context.with_session_id(session_id.0.clone());
    }
    if let Some(reason) = &request.reason {
        context = context.with_metadata("request_reason", serde_json::json!(reason));
    }
    if let Some(assignment) = &task.assignment {
        context = context
            .with_metadata("graph_id", serde_json::json!(assignment.graph_id.0))
            .with_metadata("node_id", serde_json::json!(assignment.node_id.0))
            .with_metadata("attempt", serde_json::json!(assignment.attempt));
    }
    context
}

fn skill_not_allowed_result(
    request_id: String,
    skill_id: String,
    runtime_role: &str,
) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id.clone()),
        status: SkillExecutionStatus::ValidationError,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some(format!(
            "skill '{skill_id}' is not allowed for role '{runtime_role}'"
        )),
    }
}

fn skill_executor_unavailable_result(request_id: String, skill_id: String) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id),
        status: SkillExecutionStatus::Deferred,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some("skill executor unavailable".to_string()),
    }
}

fn emit_skill_requested(
    worker_id: &AgentId,
    role: &RoleProfile,
    task: &Task,
    request_id: &str,
    request: &ParsedSkillRequest,
    event_tx: Option<&crate::agent_session::SessionEventTx>,
) {
    let (Some(event_tx), Some(session_id)) = (event_tx, &task.context.session_id) else {
        return;
    };
    let _ = event_tx.try_send(SessionEvent::SkillRequested {
        session_id: session_id.clone(),
        worker_id: WorkerId(worker_id.0.clone()),
        task_id: TaskId(task.id.clone()),
        request_id: request_id.to_string(),
        skill_id: request.skill_id.clone(),
        role: role.runtime_role.clone(),
        reason: request.reason.clone(),
    });
}

fn emit_skill_result(
    worker_id: &AgentId,
    role: &RoleProfile,
    task: &Task,
    result: &SkillExecutionResult,
    event_tx: Option<&crate::agent_session::SessionEventTx>,
) {
    let (Some(event_tx), Some(session_id)) = (event_tx, &task.context.session_id) else {
        return;
    };
    let _ = event_tx.try_send(SessionEvent::SkillResult {
        session_id: session_id.clone(),
        worker_id: WorkerId(worker_id.0.clone()),
        task_id: TaskId(task.id.clone()),
        request_id: result.request_id.clone(),
        skill_id: result.skill_id.as_str().to_string(),
        role: role.runtime_role.clone(),
        status: skill_status_name(result.status).to_string(),
        summary: result.summary.clone().map(compact_event_text),
        error: result.error.clone().map(compact_event_text),
        output: result.output.clone(),
        execution_mode: metadata_string(&result.metadata, "execution_mode"),
        risk_level: metadata_string(&result.metadata, "risk_level"),
        approval_required: result
            .metadata
            .get("approval_required")
            .and_then(serde_json::Value::as_bool),
    });
}
