use std::{
    convert::Infallible,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use agent::api::{SessionEvent, WorkerReportStatus};
use axum::{
    Extension, Json,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::Stream;
use serde_json::{Value, json};
use service::{
    api::{
        agent_context::AgentContextApi, app::AppApi, approval_audit::ApprovalAuditApi,
        briefing_ops::BriefingOpsApi, workspace::WorkspaceApi,
    },
    dto::{
        agent_context::{
            AgentContextSource, AgentRetrievalContextResponse, BuildAgentProductContextRequest,
            BuildAgentRetrievalContextRequest,
        },
        approval_audit::{CreateAuditLogRequest, CreateToolApprovalRequest},
        briefing_ops::CreateAlertRequest,
        workspace::{
            AgentWorkspaceTaskNode, BuildAgentLearningContextRequest,
            CompleteAgentWorkspaceRunRequest, EvaluateAgentWorkspaceRunRequest,
            FailAgentWorkspaceRunRequest, LinkAgentWorkspaceRunRequest,
            LinkAgentWorkspaceTasksRequest, MarkAgentWorkspaceTaskStartedRequest,
            RecordAgentSearchEvidenceRequest, RecordAgentWorkspaceTaskEvaluationRequest,
            RecordAgentWorkspaceTaskReportRequest,
        },
    },
};
use toolcraft_axum_kit::{ApiError, CommonError, middleware::auth_mw::AuthUser};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::sse::AgentStreamRequest,
    error::{Error, from_biz_error},
    statics::{
        agent_manager::{run_agent_stream, select_agent_runtime_model},
        db_manager::get_app_ctx,
    },
};

pub async fn stream_agent_session(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<AgentStreamRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    req.validate().map_err(Error::from)?;
    if req.input.trim().is_empty() {
        return Err(bad_request("input cannot be empty"));
    }
    let user_id = auth_user.user_id.clone();
    let agent_id = req
        .agent_id
        .clone()
        .or_else(|| Some("web-agent".to_string()));
    let session_id = req.session_id.clone().unwrap_or_else(new_session_id);
    let requested_workspace_id = req.workspace_id.clone();
    let input = req.input.clone();
    let (mut metadata, context_selection, model_selection) =
        req.metadata.into_parts().map_err(bad_request)?;

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(user_id.clone())
        .await
        .map_err(from_biz_error)?;
    select_agent_runtime_model(model_selection.name.as_deref())
        .map_err(|err| bad_request(err.to_string()))?;
    let context_api = AgentContextApi::new(get_app_ctx());
    if !context_selection.is_empty() {
        let context = context_api
            .build_product_context(BuildAgentProductContextRequest {
                user_id: user_id.clone(),
                product_ids: context_selection.product_ids.clone(),
                context_refs: context_selection.context_refs.clone(),
            })
            .await
            .map_err(from_biz_error)?;
        merge_agent_context_metadata(&mut metadata, context)
            .map_err(|err| Error::Custom(format!("agent context metadata failed: {err}")))?;
    }
    let retrieval_context = context_api
        .build_retrieval_context(BuildAgentRetrievalContextRequest {
            user_id: user_id.clone(),
            query: input.clone(),
            product_ids: context_selection.product_ids,
            context_refs: context_selection.context_refs,
            limit: Some(8),
        })
        .await
        .map_err(from_biz_error)?;
    merge_agent_retrieval_metadata(&mut metadata, &retrieval_context)
        .map_err(|err| Error::Custom(format!("agent retrieval metadata failed: {err}")))?;

    let workspace_api = WorkspaceApi::new(get_app_ctx());
    let linked_workspace = workspace_api
        .link_agent_run(LinkAgentWorkspaceRunRequest {
            user_id: user_id.clone(),
            requested_workspace_id,
            session_id: session_id.clone(),
            agent_id: agent_id.clone(),
            input: input.clone(),
            metadata: serde_json::to_value(&metadata).unwrap_or_else(|_| json!({})),
        })
        .await
        .map_err(from_biz_error)?;
    metadata.insert(
        "workspace.workspace_id".to_string(),
        linked_workspace.workspace.id.clone(),
    );
    metadata.insert(
        "workspace.run_id".to_string(),
        linked_workspace.run.id.clone(),
    );
    metadata.insert("workspace.source".to_string(), "agent_session".to_string());
    if let Ok(learning_context) = workspace_api
        .build_agent_learning_context(BuildAgentLearningContextRequest {
            user_id: user_id.clone(),
            role: Some("agent".to_string()),
            limit: Some(5),
        })
        .await
    {
        metadata.insert(
            "qualityLearning.trustSummary".to_string(),
            learning_context.trust_summary.to_string(),
        );
        if !learning_context.learning_summary.trim().is_empty() {
            metadata.insert(
                "qualityLearning.learningSummary".to_string(),
                learning_context.learning_summary,
            );
        }
        if !learning_context.records.is_empty()
            && let Ok(records_json) = serde_json::to_string(&learning_context.records)
        {
            metadata.insert("qualityLearning.records".to_string(), records_json);
        }
    }

    let events = run_agent_stream(
        user_id.clone(),
        Some(linked_workspace.workspace.id.clone()),
        agent_id,
        Some(session_id.clone()),
        input,
        model_selection.name,
        metadata,
    )
    .map_err(Into::<ApiError>::into)?;

    let link_state = AgentWorkspaceLinkState {
        user_id,
        session_id,
        workspace_id: linked_workspace.workspace.id.clone(),
        run_id: linked_workspace.run.id.clone(),
        latest_output: None,
        completed: false,
    };

    let stream = async_stream::stream! {
        let mut events = events;
        let workspace_api = WorkspaceApi::new(get_app_ctx());
        let approval_api = ApprovalAuditApi::new(get_app_ctx());
        let briefing_api = BriefingOpsApi::new(get_app_ctx());
        let mut link_state = link_state;
        yield Ok(typed_camel_json_event(
            "workspace_run_linked",
            serde_json::to_value(&linked_workspace)
                .unwrap_or_else(|_| json!({ "type": "workspace_run_linked" })),
        ));
        yield Ok(typed_camel_json_event(
            "rag_context_loaded",
            serde_json::to_value(&retrieval_context)
                .unwrap_or_else(|_| json!({ "type": "rag_context_loaded" })),
        ));
        while let Some(event) = events.recv().await {
            let event_name = event.event_name();
            let event_value = serde_json::to_value(&event)
                .unwrap_or_else(|_| json!({ "type": event_name }));
            yield Ok(json_event(event_name, event_value));

            match &event {
                SessionEvent::TaskGraphPlanned { graph, .. } => {
                    let response = workspace_api
                        .link_agent_tasks(LinkAgentWorkspaceTasksRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            session_id: link_state.session_id.clone(),
                            root_task: graph.root_task.clone(),
                            nodes: graph
                                .nodes
                                .iter()
                                .map(|node| AgentWorkspaceTaskNode {
                                    graph_id: graph.graph_id.0.clone(),
                                    node_id: node.id.0.clone(),
                                    worker_id: node.worker_id.0.clone(),
                                    role: node.role.clone(),
                                    title: node.title.clone(),
                                    objective: node.objective.clone(),
                                    input_refs: node
                                        .input_refs
                                        .iter()
                                        .map(|input_ref| input_ref.0.clone())
                                        .collect(),
                                    acceptance: node.acceptance.clone(),
                                    max_attempts: node.max_attempts,
                                })
                                .collect(),
                        })
                        .await;
                    match response {
                        Ok(value) => yield Ok(typed_camel_json_event(
                            "workspace_run_tasks_linked",
                            serde_json::to_value(value)
                                .unwrap_or_else(|_| json!({ "type": "workspace_run_tasks_linked" })),
                        )),
                        Err(err) => yield Ok(persistence_error_event("link_agent_tasks", err)),
                    }
                }
                SessionEvent::TaskNodeStarted {
                    graph_id,
                    node_id,
                    worker_id,
                    attempt,
                    ..
                } => {
                    let response = workspace_api
                        .mark_agent_task_started(MarkAgentWorkspaceTaskStartedRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            graph_id: graph_id.0.clone(),
                            node_id: node_id.0.clone(),
                            worker_id: worker_id.0.clone(),
                            attempt: *attempt,
                        })
                        .await;
                    match response {
                        Ok(value) => yield Ok(typed_camel_json_event(
                            "workspace_run_task_updated",
                            serde_json::to_value(value)
                                .unwrap_or_else(|_| json!({ "type": "workspace_run_task_updated" })),
                        )),
                        Err(err) => yield Ok(persistence_error_event("mark_agent_task_started", err)),
                    }
                }
                SessionEvent::TaskNodeReported { report, .. } => {
                    let response = workspace_api
                        .record_agent_task_report(RecordAgentWorkspaceTaskReportRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            graph_id: report.graph_id.0.clone(),
                            node_id: report.node_id.0.clone(),
                            task_id: report.task_id.0.clone(),
                            worker_id: report.worker_id.0.clone(),
                            role: report.role.clone(),
                            content: report.content.clone(),
                            role_output: report
                                .role_output
                                .as_ref()
                                .and_then(|role_output| serde_json::to_value(role_output).ok()),
                            evidence: report.evidence.clone(),
                            risks: report.risks.clone(),
                            open_questions: report.open_questions.clone(),
                            status: if report.status == WorkerReportStatus::Failed {
                                "failed".to_string()
                            } else {
                                "completed".to_string()
                            },
                        })
                        .await;
                    match response {
                        Ok(value) => yield Ok(typed_camel_json_event(
                            "workspace_run_task_updated",
                            serde_json::to_value(value)
                                .unwrap_or_else(|_| json!({ "type": "workspace_run_task_updated" })),
                        )),
                        Err(err) => yield Ok(persistence_error_event("record_agent_task_report", err)),
                    }
                }
                SessionEvent::TaskNodeEvaluated {
                    graph_id,
                    evaluation,
                    ..
                } => {
                    let response = workspace_api
                        .record_agent_task_evaluation(RecordAgentWorkspaceTaskEvaluationRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            graph_id: graph_id.0.clone(),
                            node_id: evaluation.node_id.0.clone(),
                            passed: evaluation.passed,
                            score: f64::from(evaluation.score),
                            reasons: evaluation.reasons.clone(),
                            rework_instruction: evaluation.rework_instruction.clone(),
                        })
                        .await;
                    match response {
                        Ok(value) => yield Ok(typed_camel_json_event(
                            "workspace_run_task_updated",
                            serde_json::to_value(value)
                                .unwrap_or_else(|_| json!({ "type": "workspace_run_task_updated" })),
                        )),
                        Err(err) => yield Ok(persistence_error_event("record_agent_task_evaluation", err)),
                    }
                }
                SessionEvent::SkillResult {
                    worker_id,
                    task_id,
                    request_id,
                    skill_id,
                    role,
                    status,
                    summary,
                    error,
                    output,
                    execution_mode,
                    risk_level,
                    approval_required,
                    ..
                } => {
                    if let Some(output) = output.clone() {
                        let response = workspace_api
                            .record_agent_search_evidence(RecordAgentSearchEvidenceRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                worker_id: worker_id.0.clone(),
                                task_id: task_id.0.clone(),
                                request_id: request_id.clone(),
                                skill_id: skill_id.clone(),
                                role: role.clone(),
                                status: status.clone(),
                                summary: summary.clone(),
                                execution_mode: execution_mode.clone(),
                                output,
                            })
                            .await;
                        match response {
                            Ok(Some(artifact)) => yield Ok(typed_camel_json_event(
                                "workspace_artifact_created",
                                serde_json::to_value(artifact)
                                    .unwrap_or_else(|_| json!({ "type": "workspace_artifact_created" })),
                            )),
                            Ok(None) => {}
                            Err(err) => {
                                yield Ok(persistence_error_event("record_agent_search_evidence", err));
                            }
                        }
                    }

                    if skill_requires_approval(status, *approval_required) {
                        let risk_level = approval_risk_level(risk_level.as_deref());
                        let reason = approval_reason(summary.as_deref());
                        let skill_result_ref = ApprovalSkillResultRef {
                            worker_id: worker_id.0.as_str(),
                            task_id: task_id.0.as_str(),
                            request_id,
                            skill_id,
                            status,
                            summary: summary.as_deref(),
                            error: error.as_deref(),
                            execution_mode: execution_mode.as_deref(),
                        };
                        let args = build_approval_args(&link_state, skill_result_ref);
                        let action_key = approval_action_key(&link_state.run_id, skill_id, request_id);
                        let approval_response = approval_api
                            .create_tool_approval(CreateToolApprovalRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: Some(link_state.workspace_id.clone()),
                                run_id: Some(link_state.run_id.clone()),
                                conversation_id: None,
                                request_role: Some(role.clone()),
                                skill_name: skill_id.clone(),
                                tool_call_id: Some(request_id.clone()),
                                risk_level: Some(risk_level.clone()),
                                args: Some(args.clone()),
                                action_key: Some(action_key.clone()),
                                reason: Some(reason.clone()),
                            })
                            .await;
                        match approval_response {
                            Ok(approval) => {
                                let audit_response = approval_api
                                    .create_audit_log(CreateAuditLogRequest {
                                        user_id: link_state.user_id.clone(),
                                        actor_user_id: None,
                                        workspace_id: Some(link_state.workspace_id.clone()),
                                        run_id: Some(link_state.run_id.clone()),
                                        execution_queue_id: None,
                                        tool_approval_id: Some(approval.id.clone()),
                                        action: "tool_approval.create".to_string(),
                                        target_type: "tool_approval".to_string(),
                                        target_id: Some(approval.id.clone()),
                                        status: Some("success".to_string()),
                                        reason: Some(reason.clone()),
                                        request_id: Some(request_id.clone()),
                                        trace_id: Some(link_state.session_id.clone()),
                                        metadata: Some(json!({
                                            "source": "agent_sse",
                                            "skillId": skill_id,
                                            "requestId": request_id,
                                            "role": role,
                                            "riskLevel": risk_level,
                                            "actionKey": action_key,
                                            "args": args,
                                        })),
                                    })
                                    .await;
                                if let Err(err) = audit_response {
                                    yield Ok(persistence_error_event("create_tool_approval_audit", err));
                                }

                                let alert_response = briefing_api
                                    .create_alert(CreateAlertRequest {
                                        user_id: link_state.user_id.clone(),
                                        workspace_id: Some(link_state.workspace_id.clone()),
                                        run_id: Some(link_state.run_id.clone()),
                                        alert_type: "agent_approval_required".to_string(),
                                        severity: Some("warning".to_string()),
                                        message: approval_alert_message(skill_id),
                                        role: Some(role.clone()),
                                        dedup_key: Some(approval_alert_dedup_key(
                                            &link_state.run_id,
                                            skill_id,
                                            request_id,
                                        )),
                                        snoozed_until: None,
                                        payload: Some(json!({
                                            "source": "agent_sse",
                                            "approvalId": approval.id,
                                            "skillId": skill_id,
                                            "requestId": request_id,
                                            "role": role,
                                            "riskLevel": risk_level,
                                            "reason": reason,
                                        })),
                                    })
                                    .await;
                                let alert = match alert_response {
                                    Ok(alert) => Some(alert),
                                    Err(err) => {
                                        yield Ok(persistence_error_event("create_approval_alert", err));
                                        None
                                    }
                                };

                                yield Ok(typed_camel_json_event(
                                    "approval_required",
                                    json!({
                                        "approval": approval,
                                        "alert": alert,
                                        "skillId": skill_id,
                                        "requestId": request_id,
                                        "role": role,
                                        "riskLevel": risk_level,
                                        "reason": reason,
                                    }),
                                ));
                            }
                            Err(err) => yield Ok(persistence_error_event("create_tool_approval", err)),
                        }
                    }
                }
                SessionEvent::LlmChunk { content, .. } => {
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::Output { content, .. } => {
                    link_state.latest_output = Some(content.clone());
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::Failed { reason, .. } => {
                    if !link_state.completed {
                        let response = workspace_api
                            .fail_agent_run(FailAgentWorkspaceRunRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                reason: reason.clone(),
                            })
                            .await;
                        match response {
                            Ok(value) => {
                                link_state.completed = true;
                                yield Ok(typed_camel_json_event(
                                    "workspace_run_failed",
                                    serde_json::to_value(value)
                                        .unwrap_or_else(|_| json!({ "type": "workspace_run_failed" })),
                                ));
                            }
                            Err(err) => yield Ok(persistence_error_event("fail_agent_run", err)),
                        }
                    }
                    yield Ok(json_event("error", json!({ "message": reason })));
                    return;
                }
                SessionEvent::Finished { .. } => {
                    if !link_state.completed {
                        let response = workspace_api
                            .complete_agent_run(CompleteAgentWorkspaceRunRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                output: link_state.latest_output.clone().unwrap_or_default(),
                            })
                            .await;
                        match response {
                            Ok(value) => {
                                link_state.completed = true;
                                yield Ok(typed_camel_json_event(
                                    "workspace_run_completed",
                                    serde_json::to_value(value.run)
                                        .unwrap_or_else(|_| json!({ "type": "workspace_run_completed" })),
                                ));
                                let artifact_id = value.artifact.as_ref().map(|artifact| artifact.id.clone());
                                if let Some(artifact) = value.artifact {
                                    yield Ok(typed_camel_json_event(
                                        "workspace_artifact_created",
                                        serde_json::to_value(artifact)
                                            .unwrap_or_else(|_| json!({ "type": "workspace_artifact_created" })),
                                    ));
                                }
                                let quality_response = workspace_api
                                    .evaluate_agent_run(EvaluateAgentWorkspaceRunRequest {
                                        user_id: link_state.user_id.clone(),
                                        workspace_id: link_state.workspace_id.clone(),
                                        run_id: link_state.run_id.clone(),
                                        artifact_id,
                                        output: link_state.latest_output.clone(),
                                    })
                                    .await;
                                match quality_response {
                                    Ok(value) => yield Ok(typed_camel_json_event(
                                        "quality_checked",
                                        serde_json::to_value(value)
                                            .unwrap_or_else(|_| json!({ "type": "quality_checked" })),
                                    )),
                                    Err(err) => yield Ok(persistence_error_event("evaluate_agent_run", err)),
                                }
                            }
                            Err(err) => yield Ok(persistence_error_event("complete_agent_run", err)),
                        }
                    }
                    yield Ok(json_event("done", json!({})));
                    return;
                }
                _ => {}
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

#[derive(Debug, Clone)]
struct AgentWorkspaceLinkState {
    user_id: String,
    session_id: String,
    workspace_id: String,
    run_id: String,
    latest_output: Option<String>,
    completed: bool,
}

#[derive(Debug, Clone, Copy)]
struct ApprovalSkillResultRef<'a> {
    worker_id: &'a str,
    task_id: &'a str,
    request_id: &'a str,
    skill_id: &'a str,
    status: &'a str,
    summary: Option<&'a str>,
    error: Option<&'a str>,
    execution_mode: Option<&'a str>,
}

fn json_event(event: &'static str, value: serde_json::Value) -> Event {
    Event::default().event(event).data(value.to_string())
}

fn typed_json_event(event: &'static str, value: Value) -> Event {
    json_event(event, ensure_event_type(event, value))
}

fn typed_camel_json_event(event: &'static str, value: Value) -> Event {
    typed_json_event(event, camelize_json_keys(value))
}

fn ensure_event_type(event: &'static str, value: Value) -> Value {
    match value {
        Value::Object(mut object) => {
            object
                .entry("type".to_string())
                .or_insert_with(|| Value::String(event.to_string()));
            Value::Object(object)
        }
        value => json!({
            "type": event,
            "data": value,
        }),
    }
}

fn camelize_json_keys(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(camelize_json_keys).collect()),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| (snake_key_to_camel(&key), camelize_json_keys(value)))
                .collect(),
        ),
        value => value,
    }
}

fn snake_key_to_camel(key: &str) -> String {
    let mut parts = key.split('_');
    let Some(first) = parts.next() else {
        return key.to_string();
    };
    let mut output = first.to_string();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            output.push(first_char.to_ascii_uppercase());
            output.extend(chars);
        }
    }
    output
}

fn persistence_error_event(action: &'static str, err: db_core::error::BizError) -> Event {
    typed_json_event(
        "workspace_run_persistence_error",
        json!({
            "action": action,
            "code": err.code(),
            "message": err.message(),
        }),
    )
}

fn skill_requires_approval(status: &str, approval_required: Option<bool>) -> bool {
    status == "pending_approval" || approval_required.unwrap_or(false)
}

fn approval_risk_level(risk_level: Option<&str>) -> String {
    match risk_level
        .unwrap_or("medium")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "low" | "l1" | "info" => "L1".to_string(),
        "medium" | "l2" | "warning" => "L2".to_string(),
        "high" | "l3" | "critical" => "L3".to_string(),
        "" => "L2".to_string(),
        value => value.to_ascii_uppercase(),
    }
}

fn approval_reason(summary: Option<&str>) -> String {
    let summary = summary.unwrap_or_default().trim();
    if summary.is_empty() {
        "Agent skill requires approval before platform write execution.".to_string()
    } else {
        summary.to_string()
    }
}

fn build_approval_args(
    link_state: &AgentWorkspaceLinkState,
    skill_result: ApprovalSkillResultRef<'_>,
) -> Value {
    json!({
        "source": "agent_sse",
        "sessionId": link_state.session_id,
        "workspaceId": link_state.workspace_id,
        "runId": link_state.run_id,
        "workerId": skill_result.worker_id,
        "taskId": skill_result.task_id,
        "requestId": skill_result.request_id,
        "skillId": skill_result.skill_id,
        "status": skill_result.status,
        "summary": skill_result.summary,
        "error": skill_result.error,
        "executionMode": skill_result.execution_mode,
    })
}

fn approval_action_key(run_id: &str, skill_id: &str, request_id: &str) -> String {
    format!(
        "agent:run:{run_id}:skill:{}:request:{}",
        compact_key_part(skill_id, 48),
        compact_key_part(request_id, 80)
    )
}

fn approval_alert_dedup_key(run_id: &str, skill_id: &str, request_id: &str) -> String {
    format!(
        "agent-approval:{run_id}:{}:{}",
        compact_key_part(skill_id, 48),
        compact_key_part(request_id, 80)
    )
}

fn compact_key_part(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '-',
        })
        .take(max_chars)
        .collect()
}

fn approval_alert_message(skill_id: &str) -> String {
    format!("Agent skill {skill_id} requires approval before execution.")
}

fn bad_request(message: impl Into<String>) -> ApiError {
    (
        axum::http::StatusCode::BAD_REQUEST,
        CommonError {
            code: db_core::error::BIZ_INTERNAL_ERROR,
            message: message.into(),
        }
        .to_json(),
    )
}

fn new_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("web-agent-{millis}-{}", Uuid::new_v4())
}

fn merge_agent_context_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: service::dto::agent_context::AgentProductContextResponse,
) -> Result<(), serde_json::Error> {
    metadata.insert(
        "context.product_ids".to_string(),
        context.product_ids.join(","),
    );
    metadata.insert("context.refs".to_string(), context.context_refs.join(","));
    metadata.insert(
        "context.product_count".to_string(),
        context.product_count.to_string(),
    );
    metadata.insert(
        "context.knowledge_count".to_string(),
        context.knowledge_count.to_string(),
    );
    metadata.insert(
        "context.source_count".to_string(),
        context.sources.len().to_string(),
    );
    if !context.summary.trim().is_empty() {
        metadata.insert("context.summary".to_string(), context.summary);
    }
    if !context.sources.is_empty() {
        metadata.insert(
            "context.sources_json".to_string(),
            serde_json::to_string(&context.sources)?,
        );
        metadata.insert(
            "context.source_titles".to_string(),
            context
                .sources
                .iter()
                .map(format_context_source)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    Ok(())
}

fn merge_agent_retrieval_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &AgentRetrievalContextResponse,
) -> Result<(), serde_json::Error> {
    metadata.insert("rag.mode".to_string(), context.mode.clone());
    metadata.insert("rag.query_terms".to_string(), context.query_terms.join(","));
    metadata.insert("rag.product_ids".to_string(), context.product_ids.join(","));
    metadata.insert(
        "rag.knowledge_count".to_string(),
        context.knowledge_count.to_string(),
    );
    metadata.insert(
        "rag.source_count".to_string(),
        context.source_count.to_string(),
    );
    metadata.insert("rag.degraded".to_string(), context.degraded.to_string());
    metadata.insert(
        "rag.external_search_available".to_string(),
        context.external_search_available.to_string(),
    );
    if let Some(reason) = &context.degraded_reason {
        metadata.insert("rag.degraded_reason".to_string(), reason.clone());
    }
    if let Some(summary) = retrieval_summary_for_agent(context) {
        metadata.insert("rag.summary".to_string(), summary.clone());
        append_context_summary(metadata, &summary);
    }
    if !context.sources.is_empty() {
        metadata.insert(
            "rag.sources_json".to_string(),
            serde_json::to_string(&context.sources)?,
        );
        merge_context_sources(metadata, &context.sources)?;
        metadata.insert(
            "rag.source_titles".to_string(),
            context
                .sources
                .iter()
                .map(format_context_source)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    Ok(())
}

fn retrieval_summary_for_agent(context: &AgentRetrievalContextResponse) -> Option<String> {
    if !context.summary.trim().is_empty() {
        return Some(context.summary.clone());
    }
    if context.degraded {
        return context.degraded_reason.as_ref().map(|reason| {
            format!(
                "Local commerce RAG retrieval found no matching product knowledge. {reason} Do not cite external search results for this turn."
            )
        });
    }
    None
}

fn append_context_summary(
    metadata: &mut std::collections::BTreeMap<String, String>,
    summary: &str,
) {
    if summary.trim().is_empty() {
        return;
    }
    metadata
        .entry("context.summary".to_string())
        .and_modify(|value| {
            if !value.trim().is_empty() {
                value.push_str("\n\n");
            }
            value.push_str("Retrieved local knowledge context:\n");
            value.push_str(summary);
        })
        .or_insert_with(|| summary.to_string());
}

fn merge_context_sources(
    metadata: &mut std::collections::BTreeMap<String, String>,
    sources: &[AgentContextSource],
) -> Result<(), serde_json::Error> {
    let mut merged_sources = metadata
        .get("context.sources_json")
        .and_then(|value| serde_json::from_str::<Vec<AgentContextSource>>(value).ok())
        .unwrap_or_default();
    for source in sources {
        if !merged_sources
            .iter()
            .any(|item| item.source_type == source.source_type && item.id == source.id)
        {
            merged_sources.push(source.clone());
        }
    }
    metadata.insert(
        "context.source_count".to_string(),
        merged_sources.len().to_string(),
    );
    metadata.insert(
        "context.sources_json".to_string(),
        serde_json::to_string(&merged_sources)?,
    );
    metadata.insert(
        "context.source_titles".to_string(),
        merged_sources
            .iter()
            .map(format_context_source)
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}

fn format_context_source(source: &AgentContextSource) -> String {
    match &source.product_id {
        Some(product_id) => format!(
            "{}:{} product={} title={}",
            source.source_type, source.id, product_id, source.title
        ),
        None => format!(
            "{}:{} title={}",
            source.source_type, source.id, source.title
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_detection_matches_skill_result_contract() {
        assert!(skill_requires_approval("pending_approval", None));
        assert!(skill_requires_approval("success", Some(true)));
        assert!(!skill_requires_approval("success", Some(false)));
        assert!(!skill_requires_approval("deferred", None));
    }

    #[test]
    fn approval_risk_level_maps_agent_values_to_business_levels() {
        assert_eq!(approval_risk_level(Some("high")), "L3");
        assert_eq!(approval_risk_level(Some("medium")), "L2");
        assert_eq!(approval_risk_level(Some("low")), "L1");
        assert_eq!(approval_risk_level(None), "L2");
    }

    #[test]
    fn approval_keys_stay_within_database_column_limits() {
        let long_skill = "platform_update_price".repeat(20);
        let long_request = "request".repeat(80);
        let action_key = approval_action_key(
            "123e4567-e89b-12d3-a456-426614174000",
            &long_skill,
            &long_request,
        );
        let dedup_key = approval_alert_dedup_key(
            "123e4567-e89b-12d3-a456-426614174000",
            &long_skill,
            &long_request,
        );

        assert!(action_key.len() <= 255);
        assert!(dedup_key.len() <= 255);
    }

    #[test]
    fn retrieval_metadata_appends_context_summary_and_sources() {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert(
            "context.summary".to_string(),
            "Selected context.".to_string(),
        );
        let source = AgentContextSource {
            source_type: "product_knowledge".to_string(),
            id: "knowledge-1".to_string(),
            product_id: Some("product-1".to_string()),
            title: "卖点".to_string(),
        };
        let context = AgentRetrievalContextResponse {
            mode: "local_product_knowledge".to_string(),
            query_terms: vec!["咖啡".to_string()],
            product_ids: vec!["product-1".to_string()],
            knowledge_count: 1,
            source_count: 1,
            summary: "RAG summary.".to_string(),
            sources: vec![source],
            degraded: false,
            degraded_reason: None,
            external_search_available: false,
        };

        merge_agent_retrieval_metadata(&mut metadata, &context).expect("metadata should merge");

        assert!(metadata["context.summary"].contains("Selected context."));
        assert!(metadata["context.summary"].contains("RAG summary."));
        assert_eq!(metadata["rag.source_count"], "1");
        assert_eq!(metadata["context.source_count"], "1");
    }

    #[test]
    fn degraded_retrieval_metadata_warns_agent_without_sources() {
        let mut metadata = std::collections::BTreeMap::new();
        let context = AgentRetrievalContextResponse {
            mode: "local_product_knowledge".to_string(),
            query_terms: vec!["库存".to_string()],
            product_ids: Vec::new(),
            knowledge_count: 0,
            source_count: 0,
            summary: String::new(),
            sources: Vec::new(),
            degraded: true,
            degraded_reason: Some("No matching local product knowledge was found; external search adapter is not configured.".to_string()),
            external_search_available: false,
        };

        merge_agent_retrieval_metadata(&mut metadata, &context)
            .expect("degraded metadata should merge");

        assert!(metadata["context.summary"].contains("no matching product knowledge"));
        assert!(metadata["context.summary"].contains("Do not cite external search results"));
        assert_eq!(metadata["rag.degraded"], "true");
    }

    #[test]
    fn skill_result_output_is_camelized_for_sse_consumers() {
        let value = typed_camel_event_value_for_test(
            "skill_result",
            json!({
                "type": "skill_result",
                "skill_id": "search_competitor",
                "execution_mode": "adapter",
                "approval_required": false,
                "output": {
                    "adapter_status": "live",
                    "planned_query": "冷萃咖啡 竞品",
                    "data": {
                        "items": [
                            {"published_at": "2026-05-10T00:00:00Z"}
                        ]
                    }
                }
            }),
        );

        assert_eq!(
            value.get("skillId").and_then(Value::as_str),
            Some("search_competitor")
        );
        assert_eq!(
            value
                .pointer("/output/adapterStatus")
                .and_then(Value::as_str),
            Some("live")
        );
        assert_eq!(
            value
                .pointer("/output/data/items/0/publishedAt")
                .and_then(Value::as_str),
            Some("2026-05-10T00:00:00Z")
        );
    }

    fn typed_camel_event_value_for_test(event: &'static str, value: Value) -> Value {
        ensure_event_type(event, camelize_json_keys(value))
    }
}
