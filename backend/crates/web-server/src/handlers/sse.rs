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
        briefing_ops::BriefingOpsApi, conversation::ConversationApi, workspace::WorkspaceApi,
    },
    dto::{
        agent_context::{
            AgentContextSource, AgentRetrievalContextResponse, BuildAgentProductContextRequest,
            BuildAgentRetrievalContextRequest,
        },
        approval_audit::{CreateAuditLogRequest, CreateToolApprovalRequest},
        briefing_ops::CreateAlertRequest,
        conversation::ConversationMessageResponse,
        workspace::{
            AgentWorkspaceTaskNode, BuildAgentLearningContextRequest,
            CompleteAgentWorkspaceRunRequest, EvaluateAgentWorkspaceRunRequest,
            FailAgentWorkspaceRunRequest, LinkAgentWorkspaceRunRequest,
            LinkAgentWorkspaceTasksRequest, MarkAgentWorkspaceTaskStartedRequest,
            MaterializeAgentCampaignDraftRequest, MaterializeAgentContentArtifactRequest,
            MaterializeAgentWorkspaceTaskRequest, RecordAgentSearchEvidenceRequest,
            RecordAgentSkillRequestedRequest, RecordAgentSkillResultRequest,
            RecordAgentWorkspaceTaskEvaluationRequest, RecordAgentWorkspaceTaskReportRequest,
            WorkspaceContextPackResponse, WorkspaceContextSourceResponse,
            WorkspaceContextSummaryResponse,
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
        external_search::get_external_search_config,
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
    let conversation_id = context_selection.conversation_id.clone();
    let selected_product_id = context_selection
        .product_ids
        .first()
        .cloned()
        .or_else(|| context_selection.first_product_ref_id());
    let product_context_refs = context_selection.product_context_refs();

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(user_id.clone())
        .await
        .map_err(from_biz_error)?;
    select_agent_runtime_model(model_selection.name.as_deref())
        .map_err(|err| bad_request(err.to_string()))?;
    if let Some(conversation_id) = conversation_id.as_ref() {
        let conversation = ConversationApi::new(get_app_ctx())
            .get_conversation(user_id.clone(), conversation_id.clone())
            .await
            .map_err(from_biz_error)?;
        merge_conversation_history_metadata(
            &mut metadata,
            conversation_id,
            &conversation.messages,
            &input,
        );
    }
    let context_api = AgentContextApi::new(get_app_ctx());
    let external_search_available = get_external_search_config().is_configured();
    if context_selection.has_product_context() {
        let context = context_api
            .build_product_context(BuildAgentProductContextRequest {
                user_id: user_id.clone(),
                product_ids: context_selection.product_ids.clone(),
                context_refs: product_context_refs.clone(),
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
            product_ids: context_selection.product_ids.clone(),
            context_refs: product_context_refs,
            allow_implicit_products: false,
            external_search_available,
            limit: Some(8),
        })
        .await
        .map_err(from_biz_error)?;
    if context_selection.has_product_context() {
        merge_agent_retrieval_metadata(&mut metadata, &retrieval_context)
            .map_err(|err| Error::Custom(format!("agent retrieval metadata failed: {err}")))?;
    }

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
    let active_run_id = Some(linked_workspace.run.id.clone());
    let workspace_context = workspace_api
        .context_summary(service::dto::workspace::GetWorkspaceContextSummaryRequest {
            user_id: user_id.clone(),
            workspace_id: linked_workspace.workspace.id.clone(),
            conversation_id,
            product_id: selected_product_id,
            include_workspace_product_context: false,
            run_id: active_run_id.clone(),
            context_refs: context_selection.context_refs.clone(),
        })
        .await
        .map_err(from_biz_error)?;
    merge_workspace_context_metadata(&mut metadata, &workspace_context)
        .map_err(|err| Error::Custom(format!("workspace context metadata failed: {err}")))?;
    let workspace_context_pack = workspace_api
        .context_pack(service::dto::workspace::GetWorkspaceContextPackRequest {
            user_id: user_id.clone(),
            workspace_id: linked_workspace.workspace.id.clone(),
            conversation_id: workspace_context.conversation_id.clone(),
            product_id: workspace_context.product_id.clone(),
            include_workspace_product_context: false,
            run_id: active_run_id,
            context_refs: context_selection.context_refs.clone(),
            max_items: Some(20),
            max_chars: Some(12_000),
        })
        .await
        .map_err(from_biz_error)?;
    merge_workspace_context_pack_metadata(&mut metadata, &workspace_context_pack)
        .map_err(|err| Error::Custom(format!("workspace context pack metadata failed: {err}")))?;
    if let Ok(learning_context) = workspace_api
        .build_agent_learning_context(BuildAgentLearningContextRequest {
            user_id: user_id.clone(),
            workspace_id: Some(linked_workspace.workspace.id.clone()),
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
            "workspace_context_loaded",
            serde_json::to_value(&workspace_context)
                .unwrap_or_else(|_| json!({ "type": "workspace_context_loaded" })),
        ));
        yield Ok(typed_camel_json_event(
            "workspace_context_pack_loaded",
            serde_json::to_value(&workspace_context_pack)
                .unwrap_or_else(|_| json!({ "type": "workspace_context_pack_loaded" })),
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
                SessionEvent::SkillRequested {
                    session_id,
                    worker_id,
                    task_id,
                    request_id,
                    skill_id,
                    role,
                    reason,
                    ..
                } => {
                    let response = workspace_api
                        .record_agent_skill_requested(RecordAgentSkillRequestedRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            session_id: Some(session_id.0.clone()),
                            worker_id: worker_id.0.clone(),
                            task_id: task_id.0.clone(),
                            request_id: request_id.clone(),
                            skill_id: skill_id.clone(),
                            role: role.clone(),
                            reason: reason.clone(),
                        })
                        .await;
                    if let Err(err) = response {
                        yield Ok(persistence_error_event("record_agent_skill_requested", err));
                    }
                }
                SessionEvent::SkillResult {
                    session_id,
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
                    let response = workspace_api
                        .record_agent_skill_result(RecordAgentSkillResultRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            session_id: Some(session_id.0.clone()),
                            worker_id: worker_id.0.clone(),
                            task_id: task_id.0.clone(),
                            request_id: request_id.clone(),
                            skill_id: skill_id.clone(),
                            role: role.clone(),
                            status: status.clone(),
                            summary: summary.clone(),
                            error: error.clone(),
                            output: output.clone(),
                            execution_mode: execution_mode.clone(),
                            risk_level: risk_level.clone(),
                            approval_required: *approval_required,
                        })
                        .await;
                    if let Err(err) = response {
                        yield Ok(persistence_error_event("record_agent_skill_result", err));
                    }

                    let task_materialize_response = workspace_api
                        .materialize_agent_workspace_task(MaterializeAgentWorkspaceTaskRequest {
                            user_id: link_state.user_id.clone(),
                            workspace_id: link_state.workspace_id.clone(),
                            run_id: link_state.run_id.clone(),
                            request_id: request_id.clone(),
                            skill_id: skill_id.clone(),
                            role: role.clone(),
                            output: output.clone().unwrap_or_else(|| json!({})),
                        })
                        .await;
                    match task_materialize_response {
                        Ok(Some(task)) => yield Ok(typed_camel_json_event(
                            "workspace_task_created",
                            serde_json::to_value(task)
                                .unwrap_or_else(|_| json!({ "type": "workspace_task_created" })),
                        )),
                        Ok(None) => {}
                        Err(err) => {
                            yield Ok(persistence_error_event("materialize_agent_workspace_task", err));
                        }
                    }

                    let memory_materialize_response = workspace_api
                        .materialize_agent_workspace_memory(
                            service::dto::workspace::MaterializeAgentWorkspaceMemoryRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                request_id: request_id.clone(),
                                skill_id: skill_id.clone(),
                                role: role.clone(),
                                output: output.clone().unwrap_or_else(|| json!({})),
                            },
                        )
                        .await;
                    match memory_materialize_response {
                        Ok(Some(record)) => yield Ok(typed_camel_json_event(
                            "agent_learning_record_created",
                            serde_json::to_value(record).unwrap_or_else(
                                |_| json!({ "type": "agent_learning_record_created" }),
                            ),
                        )),
                        Ok(None) => {}
                        Err(err) => {
                            yield Ok(persistence_error_event("materialize_agent_workspace_memory", err));
                        }
                    }

                    let campaign_materialize_response = workspace_api
                        .materialize_agent_campaign_draft(
                            MaterializeAgentCampaignDraftRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                request_id: request_id.clone(),
                                skill_id: skill_id.clone(),
                                role: role.clone(),
                                output: output.clone().unwrap_or_else(|| json!({})),
                            },
                        )
                        .await;
                    match campaign_materialize_response {
                        Ok(Some(value)) => {
                            yield Ok(typed_camel_json_event(
                                "campaign_draft_created",
                                serde_json::to_value(&value.campaign)
                                    .unwrap_or_else(|_| json!({ "type": "campaign_draft_created" })),
                            ));
                            if let Some(artifact) = value.artifact {
                                yield Ok(typed_camel_json_event(
                                    "workspace_artifact_created",
                                    serde_json::to_value(artifact).unwrap_or_else(
                                        |_| json!({ "type": "workspace_artifact_created" }),
                                    ),
                                ));
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            yield Ok(persistence_error_event("materialize_agent_campaign_draft", err));
                        }
                    }

                    let content_artifact_response = workspace_api
                        .materialize_agent_content_artifact(
                            MaterializeAgentContentArtifactRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                request_id: request_id.clone(),
                                skill_id: skill_id.clone(),
                                role: role.clone(),
                                output: output.clone().unwrap_or_else(|| json!({})),
                            },
                        )
                        .await;
                    match content_artifact_response {
                        Ok(Some(artifact)) => yield Ok(typed_camel_json_event(
                            "workspace_artifact_created",
                            serde_json::to_value(artifact)
                                .unwrap_or_else(|_| json!({ "type": "workspace_artifact_created" })),
                        )),
                        Ok(None) => {}
                        Err(err) => {
                            yield Ok(persistence_error_event("materialize_agent_content_artifact", err));
                        }
                    }

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
                            output: output.as_ref(),
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
                    yield Ok(json_event(
                        "finished",
                        json!({
                            "type": "finished",
                            "session_id": link_state.session_id,
                        }),
                    ));
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
    output: Option<&'a Value>,
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
    let normalized_inputs = skill_result
        .output
        .and_then(|output| output.get("normalized_inputs").cloned())
        .or_else(|| {
            skill_result
                .output
                .and_then(|output| output.get("normalizedInputs").cloned())
        });
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
        "normalizedInputs": normalized_inputs,
        "output": skill_result.output,
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
        metadata.insert(
            "context.selected_summary".to_string(),
            context.summary.clone(),
        );
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
                "Local commerce RAG retrieval found no matching product knowledge. {reason} Do not cite external search results unless a same-run search skill returns live sources."
            )
        });
    }
    None
}

fn append_context_summary(
    metadata: &mut std::collections::BTreeMap<String, String>,
    summary: &str,
) {
    append_context_summary_section(metadata, "Retrieved local knowledge context:", summary);
}

const CONVERSATION_HISTORY_MAX_MESSAGES: usize = 12;
const CONVERSATION_HISTORY_MAX_CHARS: usize = 6_000;
const CONVERSATION_MESSAGE_MAX_CHARS: usize = 1_200;

fn merge_conversation_history_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    conversation_id: &str,
    messages: &[ConversationMessageResponse],
    current_input: &str,
) {
    let summary = build_conversation_history_summary(messages, current_input);
    metadata.insert(
        "conversation.conversation_id".to_string(),
        conversation_id.to_string(),
    );
    metadata.insert(
        "conversation.history_message_count".to_string(),
        summary.message_count.to_string(),
    );
    if summary.text.trim().is_empty() {
        return;
    }
    metadata.insert(
        "conversation.history_summary".to_string(),
        summary.text.clone(),
    );
    append_context_summary_section(
        metadata,
        "Same conversation history context:",
        &summary.text,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConversationHistorySummary {
    message_count: usize,
    text: String,
}

fn build_conversation_history_summary(
    messages: &[ConversationMessageResponse],
    current_input: &str,
) -> ConversationHistorySummary {
    let current_input = normalize_history_text(current_input);
    let mut candidates = messages
        .iter()
        .filter_map(|message| {
            let role = conversation_history_role(&message.role)?;
            let content = normalize_history_text(message.content_text.as_deref()?);
            if content.is_empty() {
                return None;
            }
            Some((message.sequence, role, content))
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|(sequence, _, _)| *sequence);
    if let Some(index) = candidates
        .iter()
        .rposition(|(_, role, content)| *role == "User" && *content == current_input)
    {
        candidates.remove(index);
    }

    let mut selected = candidates
        .into_iter()
        .rev()
        .take(CONVERSATION_HISTORY_MAX_MESSAGES)
        .collect::<Vec<_>>();
    selected.reverse();

    let mut lines = Vec::new();
    let mut used_chars = 0usize;
    for (_, role, content) in selected {
        let content = truncate_chars(&content, CONVERSATION_MESSAGE_MAX_CHARS);
        let line = format!("{role}: {content}");
        let line_chars = line.chars().count();
        if !lines.is_empty() && used_chars + line_chars > CONVERSATION_HISTORY_MAX_CHARS {
            break;
        }
        used_chars += line_chars;
        lines.push(line);
    }

    ConversationHistorySummary {
        message_count: lines.len(),
        text: lines.join("\n"),
    }
}

fn conversation_history_role(role: &str) -> Option<&'static str> {
    match role {
        "user" => Some("User"),
        "assistant" => Some("Assistant"),
        _ => None,
    }
}

fn normalize_history_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn append_context_summary_section(
    metadata: &mut std::collections::BTreeMap<String, String>,
    heading: &str,
    summary: &str,
) {
    if summary.trim().is_empty() {
        return;
    }
    metadata
        .entry("context.combined_summary".to_string())
        .and_modify(|value| {
            if !value.trim().is_empty() {
                value.push_str("\n\n");
            }
            value.push_str(heading);
            value.push('\n');
            value.push_str(summary);
        })
        .or_insert_with(|| format!("{heading}\n{summary}"));
}

fn merge_workspace_context_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &WorkspaceContextSummaryResponse,
) -> Result<(), serde_json::Error> {
    metadata.insert(
        "workspaceContext.readiness".to_string(),
        context.readiness.clone(),
    );
    metadata.insert(
        "workspaceContext.summary".to_string(),
        context.summary.clone(),
    );
    metadata.insert(
        "workspaceContext.counts_json".to_string(),
        serde_json::to_string(&context.counts)?,
    );
    metadata.insert(
        "workspaceContext.missing_json".to_string(),
        serde_json::to_string(&context.missing)?,
    );
    metadata.insert(
        "workspaceContext.warnings_json".to_string(),
        serde_json::to_string(&context.warnings)?,
    );
    let refs = workspace_context_refs(context);
    metadata.insert("workspaceContext.refs".to_string(), refs.join(","));
    if !refs.is_empty() {
        append_context_refs(metadata, &refs);
    }
    let sources = workspace_context_sources(context);
    if !sources.is_empty() {
        metadata.insert(
            "workspaceContext.sources_json".to_string(),
            serde_json::to_string(&sources)?,
        );
        metadata.insert(
            "workspaceContext.source_titles".to_string(),
            sources
                .iter()
                .map(format_workspace_context_source)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    append_workspace_context_summary(metadata, context);
    Ok(())
}

fn merge_workspace_context_pack_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &WorkspaceContextPackResponse,
) -> Result<(), serde_json::Error> {
    metadata.insert(
        "workspaceContextPack.readiness".to_string(),
        context.readiness.clone(),
    );
    metadata.insert(
        "workspaceContextPack.summary".to_string(),
        context.summary.clone(),
    );
    metadata.insert(
        "workspaceContextPack.counts_json".to_string(),
        serde_json::to_string(&context.counts)?,
    );
    metadata.insert(
        "workspaceContextPack.items_json".to_string(),
        serde_json::to_string(&context.items)?,
    );
    metadata.insert(
        "workspaceContextPack.omitted_json".to_string(),
        serde_json::to_string(&context.omitted)?,
    );
    metadata.insert(
        "workspaceContextPack.warnings_json".to_string(),
        serde_json::to_string(&context.warnings)?,
    );
    let refs = context
        .items
        .iter()
        .map(|item| item.ref_key.clone())
        .collect::<Vec<_>>();
    metadata.insert("workspaceContextPack.refs".to_string(), refs.join(","));
    if !refs.is_empty() {
        append_context_refs(metadata, &refs);
    }
    append_workspace_context_pack_summary(metadata, context);
    Ok(())
}

fn workspace_context_refs(context: &WorkspaceContextSummaryResponse) -> Vec<String> {
    context
        .agent_context
        .get("refs")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            workspace_context_sources(context)
                .into_iter()
                .map(|source| source.ref_key.clone())
                .collect()
        })
}

fn workspace_context_sources(
    context: &WorkspaceContextSummaryResponse,
) -> Vec<WorkspaceContextSourceResponse> {
    context
        .attachment_sources
        .iter()
        .chain(context.product_asset_sources.iter())
        .chain(context.product_material_sources.iter())
        .chain(context.artifact_sources.iter())
        .cloned()
        .collect()
}

fn append_context_refs(metadata: &mut std::collections::BTreeMap<String, String>, refs: &[String]) {
    let mut merged = metadata
        .get("context.combined_refs")
        .or_else(|| metadata.get("context.refs"))
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for context_ref in refs {
        if !merged.iter().any(|item| item == context_ref) {
            merged.push(context_ref.clone());
        }
    }
    metadata.insert("context.combined_refs".to_string(), merged.join(","));
}

fn append_workspace_context_summary(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &WorkspaceContextSummaryResponse,
) {
    if context.summary.trim().is_empty() {
        return;
    }
    let refs = workspace_context_refs(context);
    let summary = if refs.is_empty() {
        context.summary.clone()
    } else {
        format!("{}\nRefs: {}", context.summary, refs.join(","))
    };
    append_context_summary_section(
        metadata,
        "Workspace file/material/artifact context:",
        &summary,
    );
}

fn append_workspace_context_pack_summary(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &WorkspaceContextPackResponse,
) {
    if context.items.is_empty() {
        return;
    }
    let mut lines = vec![context.summary.clone()];
    for item in context
        .items
        .iter()
        .filter(|item| item.text.is_some())
        .take(8)
    {
        let title = item.title.as_deref().unwrap_or("-");
        let text = item.text.as_deref().unwrap_or_default();
        lines.push(format!(
            "[{}] {} ({})\n{}",
            item.ref_key, title, item.content_kind, text
        ));
    }
    if !context.omitted.is_empty() {
        lines.push(format!("Omitted refs: {}", context.omitted.join(",")));
    }
    if !context.warnings.is_empty() {
        lines.push(format!("Warnings: {}", context.warnings.join(" | ")));
    }
    append_context_summary_section(
        metadata,
        "Workspace readable content pack:",
        &lines.join("\n\n"),
    );
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

fn format_workspace_context_source(source: &WorkspaceContextSourceResponse) -> String {
    let title = source.title.as_deref().unwrap_or("-");
    format!("{}:{} title={}", source.source_type, source.id, title)
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
            external_search_available: true,
        };

        merge_agent_retrieval_metadata(&mut metadata, &context).expect("metadata should merge");

        assert!(metadata["context.summary"].contains("Selected context."));
        assert!(metadata["context.combined_summary"].contains("RAG summary."));
        assert_eq!(metadata["rag.source_count"], "1");
        assert_eq!(metadata["rag.external_search_available"], "true");
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

        assert!(metadata["context.combined_summary"].contains("no matching product knowledge"));
        assert!(
            metadata["context.combined_summary"].contains("Do not cite external search results")
        );
        assert_eq!(metadata["rag.degraded"], "true");
        assert_eq!(metadata["rag.external_search_available"], "false");
    }

    #[test]
    fn degraded_retrieval_metadata_allows_same_run_live_search_sources() {
        let mut metadata = std::collections::BTreeMap::new();
        let context = AgentRetrievalContextResponse {
            mode: "local_product_knowledge".to_string(),
            query_terms: vec!["竞品".to_string()],
            product_ids: Vec::new(),
            knowledge_count: 0,
            source_count: 0,
            summary: String::new(),
            sources: Vec::new(),
            degraded: true,
            degraded_reason: Some("No matching local product knowledge was found; external search adapter is configured and search skills may request live sources when the task needs real-time market data.".to_string()),
            external_search_available: true,
        };

        merge_agent_retrieval_metadata(&mut metadata, &context)
            .expect("degraded metadata should merge");

        assert_eq!(metadata["rag.external_search_available"], "true");
        assert!(
            metadata["context.combined_summary"]
                .contains("unless a same-run search skill returns live sources")
        );
    }

    #[test]
    fn conversation_history_excludes_current_input() {
        let messages = vec![
            conversation_message(1, "user", "请分析咖啡类商品的营销方式"),
            conversation_message(2, "assistant", "请补充商品价格、规格和活动。"),
            conversation_message(
                3,
                "user",
                "单价100，规格每份10小包，促销活动是劳动节工作套餐",
            ),
            conversation_message(4, "user", "请直接根据前面补充的信息完成方案"),
        ];

        let summary =
            build_conversation_history_summary(&messages, "请直接根据前面补充的信息完成方案");

        assert_eq!(summary.message_count, 3);
        assert!(summary.text.contains("单价100"));
        assert!(!summary.text.contains("请直接根据前面补充的信息完成方案"));
    }

    #[test]
    fn conversation_history_metadata_appends_context_summary() {
        let messages = vec![
            conversation_message(1, "user", "前面任务"),
            conversation_message(2, "assistant", "前面回答"),
            conversation_message(3, "user", "补充：客单价100，10小包"),
        ];
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert(
            "context.summary".to_string(),
            "Selected context.".to_string(),
        );

        merge_conversation_history_metadata(
            &mut metadata,
            "conversation-1",
            &messages,
            "继续生成方案",
        );

        assert_eq!(metadata["conversation.conversation_id"], "conversation-1");
        assert_eq!(metadata["conversation.history_message_count"], "3");
        assert!(metadata["conversation.history_summary"].contains("客单价100"));
        assert!(
            metadata["context.combined_summary"].contains("Same conversation history context:")
        );
    }

    #[test]
    fn workspace_context_metadata_appends_refs_and_summary() {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("context.refs".to_string(), "product:product-1".to_string());
        metadata.insert(
            "context.summary".to_string(),
            "Selected product.".to_string(),
        );
        let context = WorkspaceContextSummaryResponse {
            workspace_id: "workspace-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            product_id: Some("product-1".to_string()),
            run_id: None,
            readiness: "ready".to_string(),
            summary: "Workspace context is ready.".to_string(),
            counts: service::dto::workspace::WorkspaceContextSummaryCounts {
                conversation_files: 1,
                product_assets: 1,
                product_materials: 0,
                workspace_artifacts: 1,
            },
            attachment_sources: vec![WorkspaceContextSourceResponse {
                source_type: "conversation_file".to_string(),
                id: "file-1".to_string(),
                title: Some("brief.pdf".to_string()),
                mime_type: Some("application/pdf".to_string()),
                file_size: Some(1024),
                status: "active".to_string(),
                ref_key: "conversation_file:file-1".to_string(),
                preview: Some("brief.pdf".to_string()),
                metadata: json!({}),
            }],
            product_asset_sources: vec![WorkspaceContextSourceResponse {
                source_type: "product_asset".to_string(),
                id: "asset-1".to_string(),
                title: Some("hero.png".to_string()),
                mime_type: Some("image/png".to_string()),
                file_size: Some(2048),
                status: "active".to_string(),
                ref_key: "product_asset:asset-1".to_string(),
                preview: Some("hero.png".to_string()),
                metadata: json!({}),
            }],
            product_material_sources: Vec::new(),
            artifact_sources: vec![WorkspaceContextSourceResponse {
                source_type: "workspace_artifact".to_string(),
                id: "artifact-1".to_string(),
                title: Some("Agent final report".to_string()),
                mime_type: Some("text/markdown".to_string()),
                file_size: None,
                status: "active".to_string(),
                ref_key: "workspace_artifact:artifact-1".to_string(),
                preview: Some("report".to_string()),
                metadata: json!({}),
            }],
            missing: Vec::new(),
            warnings: Vec::new(),
            agent_context: json!({
                "kind": "workspace_context_summary",
                "refs": [
                    "conversation_file:file-1",
                    "product_asset:asset-1",
                    "workspace_artifact:artifact-1"
                ]
            }),
        };

        merge_workspace_context_metadata(&mut metadata, &context)
            .expect("workspace context should merge");

        assert_eq!(metadata["workspaceContext.readiness"], "ready");
        assert_eq!(
            metadata["context.combined_refs"],
            "product:product-1,conversation_file:file-1,product_asset:asset-1,workspace_artifact:artifact-1"
        );
        assert!(metadata["context.summary"].contains("Selected product."));
        assert!(
            metadata["context.combined_summary"]
                .contains("Workspace file/material/artifact context:")
        );
        assert!(metadata["workspaceContext.sources_json"].contains("conversation_file"));
    }

    #[test]
    fn workspace_context_pack_metadata_appends_readable_text() {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert(
            "context.summary".to_string(),
            "Selected product.".to_string(),
        );
        let context = WorkspaceContextPackResponse {
            workspace_id: "workspace-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            product_id: Some("product-1".to_string()),
            run_id: None,
            readiness: "ready".to_string(),
            summary: "Workspace context pack is ready.".to_string(),
            counts: service::dto::workspace::WorkspaceContextSummaryCounts {
                conversation_files: 1,
                product_assets: 0,
                product_materials: 1,
                workspace_artifacts: 0,
            },
            items: vec![
                service::dto::workspace::WorkspaceContextPackItemResponse {
                    source_type: "product_material".to_string(),
                    id: "material-1".to_string(),
                    title: Some("卖点".to_string()),
                    ref_key: "product_material:material-1".to_string(),
                    content_kind: "content".to_string(),
                    text: Some("主打冷萃低糖。".to_string()),
                    text_chars: 7,
                    truncated: false,
                    metadata: json!({}),
                },
                service::dto::workspace::WorkspaceContextPackItemResponse {
                    source_type: "conversation_file".to_string(),
                    id: "file-1".to_string(),
                    title: Some("brief.pdf".to_string()),
                    ref_key: "conversation_file:file-1".to_string(),
                    content_kind: "metadata_only".to_string(),
                    text: None,
                    text_chars: 0,
                    truncated: false,
                    metadata: json!({}),
                },
            ],
            omitted: Vec::new(),
            warnings: vec![
                "1 conversation file(s) only have metadata; binary content was not read"
                    .to_string(),
            ],
            agent_context: json!({"kind": "workspace_context_pack"}),
        };

        merge_workspace_context_pack_metadata(&mut metadata, &context)
            .expect("workspace context pack should merge");

        assert_eq!(metadata["workspaceContextPack.readiness"], "ready");
        assert_eq!(
            metadata["context.combined_refs"],
            "product_material:material-1,conversation_file:file-1"
        );
        assert!(metadata["context.combined_summary"].contains("Workspace readable content pack:"));
        assert!(metadata["context.combined_summary"].contains("主打冷萃低糖"));
        assert!(metadata["workspaceContextPack.items_json"].contains("product_material"));
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

    fn conversation_message(
        sequence: i64,
        role: &str,
        content: &str,
    ) -> ConversationMessageResponse {
        ConversationMessageResponse {
            id: format!("message-{sequence}"),
            conversation_id: "conversation-1".to_string(),
            user_id: "user-1".to_string(),
            role: role.to_string(),
            content_text: Some(content.to_string()),
            content_json: None,
            status: "active".to_string(),
            sequence,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
        }
    }
}
