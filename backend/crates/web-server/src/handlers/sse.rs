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
use serde::Serialize;
use serde_json::{Value, json};
use service::{
    api::{
        agent_context::AgentContextApi, app::AppApi, approval_audit::ApprovalAuditApi,
        briefing_ops::BriefingOpsApi, conversation::ConversationApi,
        vision_preprocess_manager::VisionPreprocessManager, workspace::WorkspaceApi,
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
use utils::image::is_supported_vision_mime;
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

const MAX_AUTO_VISION_IMAGES: usize = 4;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentVisionImageSummaryAttachment {
    source_ref: String,
    object_key: String,
    summary: String,
    agent_context: Option<String>,
    analysis_id: String,
    prompt_version: Option<String>,
    llm_name: Option<String>,
}

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
    let input_for_memory = input.clone();
    let (mut metadata, context_selection, model_selection) =
        req.metadata.into_parts().map_err(bad_request)?;
    let conversation_id = context_selection.conversation_id.clone();
    let mut conversation_memory_summary: Option<Value> = None;
    let mut conversation_context_loaded: Option<Value> = None;
    let selected_product_id = context_selection
        .product_ids
        .first()
        .cloned()
        .or_else(|| context_selection.first_product_ref_id());
    let mut workspace_context_refs = context_selection.context_refs.clone();
    for campaign_id in &context_selection.campaign_ids {
        let campaign_ref = format!("campaign:{campaign_id}");
        if !workspace_context_refs
            .iter()
            .any(|item| item == &campaign_ref)
        {
            workspace_context_refs.push(campaign_ref);
        }
    }
    for campaign_ref in context_selection.campaign_context_refs() {
        if !workspace_context_refs
            .iter()
            .any(|item| item == &campaign_ref)
        {
            workspace_context_refs.push(campaign_ref);
        }
    }
    let product_context_refs = context_selection.product_context_refs();

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(user_id.clone())
        .await
        .map_err(from_biz_error)?;
    select_agent_runtime_model(model_selection.name.as_deref())
        .map_err(|err| bad_request(err.to_string()))?;
    if let Some(conversation_id) = conversation_id.as_ref() {
        let conversation_api = ConversationApi::new(get_app_ctx());
        let conversation = conversation_api
            .get_conversation(user_id.clone(), conversation_id.clone())
            .await
            .map_err(from_biz_error)?;
        let previous_memory = conversation_api
            .get_conversation_memory(user_id.clone(), conversation_id.clone())
            .await
            .map_err(from_biz_error)?;
        let conversation_history_summary = merge_conversation_history_metadata(
            &mut metadata,
            conversation_id,
            &conversation.messages,
            &input,
            previous_memory.as_ref(),
        );
        conversation_context_loaded = Some(build_conversation_context_loaded_event(
            conversation_id,
            &conversation_history_summary,
            previous_memory.is_some(),
        ));
        if conversation_history_summary.total_message_count > 0 {
            conversation_memory_summary = Some(conversation_history_summary.memory_json.clone());
        }
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
            conversation_id: conversation_id.clone(),
            product_id: selected_product_id,
            include_workspace_product_context: false,
            run_id: active_run_id.clone(),
            context_refs: workspace_context_refs.clone(),
        })
        .await
        .map_err(from_biz_error)?;
    tracing::debug!(
        conversation_id = ?workspace_context.conversation_id,
        attachment_sources = workspace_context.attachment_sources.len(),
        "sse workspace context summary loaded"
    );
    merge_workspace_context_metadata(&mut metadata, &workspace_context)
        .map_err(|err| Error::Custom(format!("workspace context metadata failed: {err}")))?;
    merge_conversation_file_image_attachments(&mut metadata, &user_id, &workspace_context)
        .await
        .map_err(Into::<ApiError>::into)?;
    let workspace_context_pack = workspace_api
        .context_pack(service::dto::workspace::GetWorkspaceContextPackRequest {
            user_id: user_id.clone(),
            workspace_id: linked_workspace.workspace.id.clone(),
            conversation_id: workspace_context.conversation_id.clone(),
            product_id: workspace_context.product_id.clone(),
            include_workspace_product_context: false,
            run_id: active_run_id,
            context_refs: workspace_context_refs,
            max_items: Some(20),
            max_chars: Some(12_000),
            query: Some(input.clone()),
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
        conversation_id,
        input: input_for_memory,
        conversation_memory_summary,
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
        if let Some(conversation_context_loaded) = conversation_context_loaded {
            yield Ok(typed_camel_json_event(
                "conversation_context_loaded",
                conversation_context_loaded,
            ));
        }
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
                                if let Some(conversation_id) = link_state.conversation_id.clone()
                                    && let Some(memory) =
                                        build_completed_conversation_memory(&link_state)
                                {
                                    let response = ConversationApi::new(get_app_ctx())
                                        .update_conversation_memory(
                                            service::dto::conversation::UpdateConversationMemoryRequest {
                                                user_id: link_state.user_id.clone(),
                                                conversation_id,
                                                memory,
                                            },
                                        )
                                        .await;
                                    match response {
                                        Ok(value) => yield Ok(typed_camel_json_event(
                                            "conversation_memory_updated",
                                            serde_json::to_value(value).unwrap_or_else(
                                                |_| json!({ "type": "conversation_memory_updated" }),
                                            ),
                                        )),
                                        Err(err) => {
                                            yield Ok(persistence_error_event(
                                                "update_conversation_memory",
                                                err,
                                            ));
                                        }
                                    }
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
    conversation_id: Option<String>,
    input: String,
    conversation_memory_summary: Option<Value>,
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

fn build_completed_conversation_memory(link_state: &AgentWorkspaceLinkState) -> Option<Value> {
    let latest_output =
        normalize_history_text(link_state.latest_output.as_deref().unwrap_or_default());
    if latest_output.is_empty() && link_state.conversation_memory_summary.is_none() {
        return None;
    }

    let mut memory = link_state
        .conversation_memory_summary
        .clone()
        .filter(Value::is_object)
        .unwrap_or_else(|| {
            json!({
                "kind": "conversation_memory_summary",
                "rollingSummary": "No older compressed history is needed for this turn.",
                "facts": [],
                "decisions": [],
                "constraints": [],
                "openQuestions": [],
                "sourceMap": {}
            })
        });
    if let Some(object) = memory.as_object_mut() {
        object.insert(
            "lastUserInput".to_string(),
            Value::String(truncate_chars(
                &normalize_history_text(&link_state.input),
                CONVERSATION_MEMORY_LAST_USER_INPUT_MAX_CHARS,
            )),
        );
        if !latest_output.is_empty() {
            object.insert(
                "lastAssistantOutput".to_string(),
                Value::String(truncate_chars(
                    &latest_output,
                    CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS,
                )),
            );
            upsert_memory_array_item(
                object,
                "decisions",
                json!({
                    "source": format!("agent_session:{}", link_state.session_id),
                    "text": truncate_chars(&latest_output, 220),
                    "confidence": "generated",
                }),
                CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS,
            );
        }
        object.insert(
            "updatedAt".to_string(),
            Value::String(current_unix_millis().to_string()),
        );
        if let Some(source_map) = object
            .entry("sourceMap".to_string())
            .or_insert_with(|| json!({}))
            .as_object_mut()
        {
            source_map.insert(
                "lastSessionId".to_string(),
                Value::String(link_state.session_id.clone()),
            );
            source_map.insert("persisted".to_string(), Value::Bool(true));
        }
    }
    Some(compact_conversation_memory_json(memory))
}

fn upsert_memory_array_item(
    object: &mut serde_json::Map<String, Value>,
    key: &str,
    item: Value,
    max_items: usize,
) {
    let mut items = object
        .remove(key)
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    items.insert(0, item);
    items.truncate(max_items);
    object.insert(key.to_string(), Value::Array(items));
}

fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
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
            'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '-' | '_' | '.' => ch,
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
                "Local commerce RAG retrieval found no matching product knowledge. {reason} Do \
                 not cite external search results unless a same-run search skill returns live \
                 sources."
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

const CONVERSATION_HISTORY_RECENT_MESSAGES: usize = 12;
const CONVERSATION_HISTORY_MAX_CHARS: usize = 6_000;
const CONVERSATION_MESSAGE_MAX_CHARS: usize = 1_200;
const CONVERSATION_HISTORY_OLDER_SUMMARY_MESSAGES: usize = 8;
const CONVERSATION_HISTORY_OLDER_SUMMARY_MAX_CHARS: usize = 1_600;
const CONVERSATION_HISTORY_RELEVANT_OLDER_MESSAGES: usize = 4;
const CONVERSATION_MEMORY_ROLLING_SUMMARY_MAX_CHARS: usize = 1_400;
const CONVERSATION_MEMORY_ITEM_MAX_CHARS: usize = 260;
const CONVERSATION_MEMORY_SOURCE_MAX_CHARS: usize = 120;
const CONVERSATION_MEMORY_CONFIDENCE_MAX_CHARS: usize = 40;
const CONVERSATION_MEMORY_FACTS_MAX_ITEMS: usize = 10;
const CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS: usize = 6;
const CONVERSATION_MEMORY_CONSTRAINTS_MAX_ITEMS: usize = 8;
const CONVERSATION_MEMORY_OPEN_QUESTIONS_MAX_ITEMS: usize = 5;
const CONVERSATION_MEMORY_LAST_USER_INPUT_MAX_CHARS: usize = 500;
const CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS: usize = 700;
const CONVERSATION_MEMORY_TEXT_MAX_CHARS: usize = 2_400;

fn merge_conversation_history_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    conversation_id: &str,
    messages: &[ConversationMessageResponse],
    current_input: &str,
    previous_memory: Option<&Value>,
) -> ConversationHistorySummary {
    let summary = build_conversation_history_summary(messages, current_input, previous_memory);
    metadata.insert(
        "conversation.conversation_id".to_string(),
        conversation_id.to_string(),
    );
    metadata.insert(
        "conversation.history_message_count".to_string(),
        summary.message_count.to_string(),
    );
    metadata.insert(
        "conversation.history_total_message_count".to_string(),
        summary.total_message_count.to_string(),
    );
    metadata.insert(
        "conversation.history_truncated".to_string(),
        summary.truncated.to_string(),
    );
    metadata.insert(
        "conversation.history_relevant_older_count".to_string(),
        summary.relevant_older_count.to_string(),
    );
    if !summary.older_summary.trim().is_empty() {
        metadata.insert(
            "conversation.history_older_summary".to_string(),
            summary.older_summary.clone(),
        );
    }
    if summary.total_message_count > 0 {
        metadata.insert(
            "conversation.memory_summary_json".to_string(),
            summary.memory_json.to_string(),
        );
        append_context_summary_section(
            metadata,
            "Same conversation structured memory:",
            &summary.memory_text,
        );
    }
    if summary.text.trim().is_empty() {
        return summary;
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
    summary
}

fn build_conversation_context_loaded_event(
    conversation_id: &str,
    summary: &ConversationHistorySummary,
    previous_memory_available: bool,
) -> Value {
    json!({
        "conversationId": conversation_id,
        "readiness": if summary.total_message_count > 0 { "ready" } else { "empty" },
        "messageCount": summary.message_count,
        "totalMessageCount": summary.total_message_count,
        "truncated": summary.truncated,
        "relevantOlderCount": summary.relevant_older_count,
        "olderSummaryAvailable": !summary.older_summary.trim().is_empty(),
        "memoryAvailable": previous_memory_available || summary.total_message_count > 0,
        "previousMemoryAvailable": previous_memory_available,
        "historyChars": summary.text.chars().count(),
        "memoryChars": summary.memory_text.chars().count(),
        "summary": build_conversation_context_loaded_summary(summary, previous_memory_available),
        "memory": {
            "kind": summary.memory_json.get("kind").cloned().unwrap_or(Value::Null),
            "rollingSummary": summary.memory_json
                .get("rollingSummary")
                .and_then(Value::as_str)
                .map(|value| truncate_chars(value, 500)),
            "factCount": summary.memory_json
                .get("facts")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default(),
            "decisionCount": summary.memory_json
                .get("decisions")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default(),
            "constraintCount": summary.memory_json
                .get("constraints")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default(),
            "openQuestionCount": summary.memory_json
                .get("openQuestions")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default(),
        }
    })
}

fn build_conversation_context_loaded_summary(
    summary: &ConversationHistorySummary,
    previous_memory_available: bool,
) -> String {
    if summary.total_message_count == 0 {
        return "No same-conversation history was available for this turn.".to_string();
    }
    format!(
        "Same conversation context loaded: {}/{} recent message(s), truncated={}, relevant older \
         message(s)={}, previous memory={}.",
        summary.message_count,
        summary.total_message_count,
        summary.truncated,
        summary.relevant_older_count,
        previous_memory_available
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConversationHistorySummary {
    pub(crate) message_count: usize,
    pub(crate) total_message_count: usize,
    pub(crate) truncated: bool,
    pub(crate) relevant_older_count: usize,
    pub(crate) older_summary: String,
    pub(crate) memory_json: Value,
    pub(crate) memory_text: String,
    pub(crate) text: String,
}

pub(crate) fn build_conversation_history_summary(
    messages: &[ConversationMessageResponse],
    current_input: &str,
    previous_memory: Option<&Value>,
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

    let total_message_count = candidates.len();
    let truncated = total_message_count > CONVERSATION_HISTORY_RECENT_MESSAGES;
    let recent_start = total_message_count.saturating_sub(CONVERSATION_HISTORY_RECENT_MESSAGES);
    let older_summary = build_older_conversation_summary(&candidates[.. recent_start]);
    let relevant_older =
        select_relevant_older_conversation_messages(&candidates[.. recent_start], &current_input);
    let memory = build_conversation_memory_summary(&candidates, &older_summary, previous_memory);

    let selected = candidates
        .into_iter()
        .skip(recent_start)
        .collect::<Vec<_>>();
    let selected_count = selected.len();

    let mut lines = Vec::new();
    if !older_summary.is_empty() {
        lines.push(format!("Earlier summary: {older_summary}"));
    }
    if !relevant_older.is_empty() {
        lines.push("Relevant earlier messages:".to_string());
        lines.extend(
            relevant_older
                .iter()
                .map(|(_, role, content)| format!("- {role}: {}", truncate_chars(content, 360))),
        );
    }
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
        message_count: selected_count,
        total_message_count,
        truncated,
        relevant_older_count: relevant_older.len(),
        older_summary,
        memory_text: memory_text_from_json(&memory),
        memory_json: memory,
        text: lines.join("\n"),
    }
}

fn select_relevant_older_conversation_messages(
    candidates: &[(i64, &'static str, String)],
    current_input: &str,
) -> Vec<(i64, &'static str, String)> {
    let query_terms = tokenize_history_query(current_input);
    if query_terms.is_empty() || candidates.is_empty() {
        return Vec::new();
    }
    let mut scored = candidates
        .iter()
        .map(|(sequence, role, content)| {
            (
                *sequence,
                *role,
                content.clone(),
                score_history_text(content, &query_terms),
            )
        })
        .filter(|(_, _, _, score)| *score > 0)
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.3.cmp(&left.3).then_with(|| left.0.cmp(&right.0)));
    scored.truncate(CONVERSATION_HISTORY_RELEVANT_OLDER_MESSAGES);
    scored.sort_by_key(|(sequence, _, _, _)| *sequence);
    scored
        .into_iter()
        .map(|(sequence, role, content, _)| (sequence, role, content))
        .collect()
}

fn score_history_text(text: &str, query_terms: &[String]) -> usize {
    let normalized = normalize_history_text(text).to_ascii_lowercase();
    query_terms
        .iter()
        .map(|term| normalized.matches(term).count() * term.chars().count().max(1))
        .sum()
}

fn tokenize_history_query(query: &str) -> Vec<String> {
    let normalized = normalize_history_text(query).to_ascii_lowercase();
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut current_is_cjk = false;
    for ch in normalized.chars() {
        if ch.is_ascii_alphanumeric() {
            if current_is_cjk {
                add_history_query_token(&mut terms, &current, true);
                current.clear();
            }
            current_is_cjk = false;
            current.push(ch);
        } else if is_cjk(ch) {
            if !current.is_empty() && !current_is_cjk {
                add_history_query_token(&mut terms, &current, false);
                current.clear();
            }
            current_is_cjk = true;
            current.push(ch);
        } else if !current.is_empty() {
            add_history_query_token(&mut terms, &current, current_is_cjk);
            current.clear();
        }
        if terms.len() >= 24 {
            break;
        }
    }
    if !current.is_empty() && terms.len() < 24 {
        add_history_query_token(&mut terms, &current, current_is_cjk);
    }
    terms
}

fn add_history_query_token(terms: &mut Vec<String>, token: &str, cjk: bool) {
    if cjk {
        let chars = token.chars().collect::<Vec<_>>();
        if chars.len() == 2 {
            push_history_query_term(terms, token.to_string());
        } else if chars.len() > 2 {
            for window in chars.windows(2) {
                push_history_query_term(terms, window.iter().collect::<String>());
                if terms.len() >= 24 {
                    return;
                }
            }
            for window in chars.windows(3) {
                push_history_query_term(terms, window.iter().collect::<String>());
                if terms.len() >= 24 {
                    return;
                }
            }
        }
        return;
    }
    if token.len() > 1 {
        push_history_query_term(terms, token.to_string());
    }
}

fn push_history_query_term(terms: &mut Vec<String>, term: String) {
    if !term.trim().is_empty() && !terms.iter().any(|existing| existing == &term) {
        terms.push(term);
    }
}

fn is_cjk(ch: char) -> bool {
    ('\u{4e00}' ..= '\u{9fff}').contains(&ch)
        || ('\u{3400}' ..= '\u{4dbf}').contains(&ch)
        || ('\u{f900}' ..= '\u{faff}').contains(&ch)
}

fn build_conversation_memory_summary(
    candidates: &[(i64, &'static str, String)],
    older_summary: &str,
    previous_memory: Option<&Value>,
) -> Value {
    let recent_user_messages = candidates
        .iter()
        .rev()
        .filter(|(_, role, _)| *role == "User")
        .take(6)
        .map(|(sequence, _, content)| {
            json!({
                "source": format!("conversation_message:{sequence}"),
                "text": truncate_chars(content, 240),
                "confidence": "explicit",
            })
        })
        .collect::<Vec<_>>();
    let recent_assistant_notes = candidates
        .iter()
        .rev()
        .filter(|(_, role, _)| *role == "Assistant")
        .take(3)
        .map(|(sequence, _, content)| {
            json!({
                "source": format!("conversation_message:{sequence}"),
                "text": truncate_chars(content, 220),
                "confidence": "generated",
            })
        })
        .collect::<Vec<_>>();
    let previous_facts = previous_memory_array(
        previous_memory,
        "facts",
        CONVERSATION_MEMORY_FACTS_MAX_ITEMS,
    );
    let previous_decisions = previous_memory_array(
        previous_memory,
        "decisions",
        CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS,
    );
    let previous_constraints = previous_memory_array(
        previous_memory,
        "constraints",
        CONVERSATION_MEMORY_CONSTRAINTS_MAX_ITEMS,
    );
    let previous_open_questions = previous_memory_array(
        previous_memory,
        "openQuestions",
        CONVERSATION_MEMORY_OPEN_QUESTIONS_MAX_ITEMS,
    );
    let previous_rolling_summary = previous_memory
        .and_then(|value| value.get("rollingSummary"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let rolling_summary = if !older_summary.trim().is_empty() {
        match previous_rolling_summary {
            Some(previous)
                if previous != "No older compressed history is needed for this turn." =>
            {
                truncate_chars(
                    &format!("{previous}\n{older_summary}"),
                    CONVERSATION_MEMORY_ROLLING_SUMMARY_MAX_CHARS,
                )
            }
            _ => older_summary.to_string(),
        }
    } else {
        previous_rolling_summary
            .unwrap_or("No older compressed history is needed for this turn.")
            .to_string()
    };

    compact_conversation_memory_json(json!({
        "kind": "conversation_memory_summary",
        "rollingSummary": rolling_summary,
        "facts": merge_memory_items(
            recent_user_messages,
            previous_facts,
            CONVERSATION_MEMORY_FACTS_MAX_ITEMS,
        ),
        "decisions": merge_memory_items(
            recent_assistant_notes,
            previous_decisions,
            CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS,
        ),
        "constraints": merge_memory_items(
            extract_history_constraints(candidates),
            previous_constraints,
            CONVERSATION_MEMORY_CONSTRAINTS_MAX_ITEMS,
        ),
        "openQuestions": previous_open_questions,
        "sourceMap": {
            "messageCount": candidates.len(),
            "recentWindow": CONVERSATION_HISTORY_RECENT_MESSAGES,
            "previousMemoryAvailable": previous_memory.is_some(),
        }
    }))
}

fn previous_memory_array(
    previous_memory: Option<&Value>,
    key: &str,
    max_items: usize,
) -> Vec<Value> {
    previous_memory
        .and_then(|value| value.get(key))
        .and_then(Value::as_array)
        .map(|items| items.iter().take(max_items).cloned().collect())
        .unwrap_or_default()
}

fn merge_memory_items(primary: Vec<Value>, secondary: Vec<Value>, max_items: usize) -> Vec<Value> {
    let mut output = Vec::new();
    for item in primary.into_iter().chain(secondary) {
        let Some(item) = sanitize_memory_item(&item, "previous") else {
            continue;
        };
        if !upsert_compact_memory_item(&mut output, item, max_items) {
            continue;
        }
    }
    output
}

fn compact_conversation_memory_json(memory: Value) -> Value {
    let object = memory.as_object();
    let rolling_summary = object
        .and_then(|object| object.get("rollingSummary"))
        .and_then(Value::as_str)
        .map(normalize_history_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "No older compressed history is needed for this turn.".to_string());
    let mut compacted = serde_json::Map::new();
    compacted.insert(
        "kind".to_string(),
        Value::String("conversation_memory_summary".to_string()),
    );
    compacted.insert(
        "rollingSummary".to_string(),
        Value::String(truncate_chars(
            &rolling_summary,
            CONVERSATION_MEMORY_ROLLING_SUMMARY_MAX_CHARS,
        )),
    );
    compacted.insert(
        "facts".to_string(),
        Value::Array(compact_memory_array(
            object.and_then(|object| object.get("facts")),
            CONVERSATION_MEMORY_FACTS_MAX_ITEMS,
            "explicit",
        )),
    );
    compacted.insert(
        "decisions".to_string(),
        Value::Array(compact_memory_array(
            object.and_then(|object| object.get("decisions")),
            CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS,
            "generated",
        )),
    );
    compacted.insert(
        "constraints".to_string(),
        Value::Array(compact_memory_array(
            object.and_then(|object| object.get("constraints")),
            CONVERSATION_MEMORY_CONSTRAINTS_MAX_ITEMS,
            "explicit",
        )),
    );
    compacted.insert(
        "openQuestions".to_string(),
        Value::Array(compact_open_questions(
            object.and_then(|object| object.get("openQuestions")),
            CONVERSATION_MEMORY_OPEN_QUESTIONS_MAX_ITEMS,
        )),
    );
    if let Some(value) = compact_optional_string_field(
        object,
        "lastUserInput",
        CONVERSATION_MEMORY_LAST_USER_INPUT_MAX_CHARS,
    ) {
        compacted.insert("lastUserInput".to_string(), Value::String(value));
    }
    if let Some(value) = compact_optional_string_field(
        object,
        "lastAssistantOutput",
        CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS,
    ) {
        compacted.insert("lastAssistantOutput".to_string(), Value::String(value));
    }
    if let Some(value) = compact_optional_string_field(object, "updatedAt", 40) {
        compacted.insert("updatedAt".to_string(), Value::String(value));
    }
    compacted.insert(
        "sourceMap".to_string(),
        compact_memory_source_map(object.and_then(|object| object.get("sourceMap"))),
    );
    Value::Object(compacted)
}

fn compact_memory_array(
    value: Option<&Value>,
    max_items: usize,
    default_confidence: &str,
) -> Vec<Value> {
    let mut output = Vec::new();
    let Some(items) = value.and_then(Value::as_array) else {
        return output;
    };
    for item in items {
        let Some(item) = sanitize_memory_item(item, default_confidence) else {
            continue;
        };
        if !upsert_compact_memory_item(&mut output, item, max_items) {
            continue;
        }
    }
    output
}

fn upsert_compact_memory_item(output: &mut Vec<Value>, item: Value, max_items: usize) -> bool {
    let item_key = memory_item_dedupe_key(&item);
    if item_key.is_empty() {
        return false;
    }
    if let Some(existing_index) = output
        .iter()
        .position(|existing| memory_items_overlap(existing, &item_key))
    {
        let existing_len = output[existing_index]
            .get("text")
            .and_then(Value::as_str)
            .map(|value| value.chars().count())
            .unwrap_or_default();
        let item_len = item
            .get("text")
            .and_then(Value::as_str)
            .map(|value| value.chars().count())
            .unwrap_or_default();
        if item_len > existing_len {
            output[existing_index] = item;
        }
        return false;
    }
    if output.len() >= max_items {
        return false;
    }
    output.push(item);
    true
}

fn sanitize_memory_item(item: &Value, default_confidence: &str) -> Option<Value> {
    let object = item.as_object();
    let text = item
        .as_str()
        .or_else(|| {
            object
                .and_then(|object| object.get("text"))
                .and_then(Value::as_str)
        })
        .map(normalize_history_text)
        .filter(|value| !value.trim().is_empty())?;
    let source = object
        .and_then(|object| object.get("source"))
        .and_then(Value::as_str)
        .map(normalize_history_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "previous_memory".to_string());
    let confidence = object
        .and_then(|object| object.get("confidence"))
        .and_then(Value::as_str)
        .map(normalize_history_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_confidence.to_string());
    Some(json!({
        "source": truncate_chars(&source, CONVERSATION_MEMORY_SOURCE_MAX_CHARS),
        "text": truncate_chars(&text, CONVERSATION_MEMORY_ITEM_MAX_CHARS),
        "confidence": truncate_chars(&confidence, CONVERSATION_MEMORY_CONFIDENCE_MAX_CHARS),
    }))
}

fn memory_item_dedupe_key(item: &Value) -> String {
    item.get("text")
        .and_then(Value::as_str)
        .map(normalize_history_text)
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn memory_items_overlap(existing: &Value, item_key: &str) -> bool {
    let existing_key = memory_item_dedupe_key(existing);
    if existing_key.is_empty() || item_key.is_empty() {
        return false;
    }
    existing_key == item_key
        || (existing_key.chars().count() >= 8 && item_key.contains(&existing_key))
        || (item_key.chars().count() >= 8 && existing_key.contains(item_key))
}

fn compact_open_questions(value: Option<&Value>, max_items: usize) -> Vec<Value> {
    let mut output = Vec::new();
    let mut seen = Vec::<String>::new();
    let Some(items) = value.and_then(Value::as_array) else {
        return output;
    };
    for item in items {
        let compacted = if let Some(question) = item.as_str() {
            let question = normalize_history_text(question);
            if question.is_empty() {
                None
            } else {
                Some(Value::String(truncate_chars(
                    &question,
                    CONVERSATION_MEMORY_ITEM_MAX_CHARS,
                )))
            }
        } else {
            sanitize_memory_item(item, "open")
        };
        let Some(compacted) = compacted else {
            continue;
        };
        let key = memory_open_question_key(&compacted);
        if key.is_empty() || seen.iter().any(|existing| existing == &key) {
            continue;
        }
        seen.push(key);
        output.push(compacted);
        if output.len() >= max_items {
            break;
        }
    }
    output
}

fn memory_open_question_key(item: &Value) -> String {
    item.as_str()
        .or_else(|| item.get("text").and_then(Value::as_str))
        .map(normalize_history_text)
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn compact_optional_string_field(
    object: Option<&serde_json::Map<String, Value>>,
    key: &str,
    max_chars: usize,
) -> Option<String> {
    object
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .map(normalize_history_text)
        .filter(|value| !value.trim().is_empty())
        .map(|value| truncate_chars(&value, max_chars))
}

fn compact_memory_source_map(value: Option<&Value>) -> Value {
    let mut source_map = serde_json::Map::new();
    if let Some(object) = value.and_then(Value::as_object) {
        copy_memory_source_map_field(object, &mut source_map, "messageCount");
        copy_memory_source_map_field(object, &mut source_map, "recentWindow");
        copy_memory_source_map_field(object, &mut source_map, "previousMemoryAvailable");
        copy_memory_source_map_field(object, &mut source_map, "lastSessionId");
        copy_memory_source_map_field(object, &mut source_map, "persisted");
    }
    source_map.insert(
        "memoryBudget".to_string(),
        json!({
            "rollingSummaryMaxChars": CONVERSATION_MEMORY_ROLLING_SUMMARY_MAX_CHARS,
            "itemMaxChars": CONVERSATION_MEMORY_ITEM_MAX_CHARS,
            "factsMaxItems": CONVERSATION_MEMORY_FACTS_MAX_ITEMS,
            "decisionsMaxItems": CONVERSATION_MEMORY_DECISIONS_MAX_ITEMS,
            "constraintsMaxItems": CONVERSATION_MEMORY_CONSTRAINTS_MAX_ITEMS,
            "openQuestionsMaxItems": CONVERSATION_MEMORY_OPEN_QUESTIONS_MAX_ITEMS,
            "lastUserInputMaxChars": CONVERSATION_MEMORY_LAST_USER_INPUT_MAX_CHARS,
            "lastAssistantOutputMaxChars": CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS,
            "memoryTextMaxChars": CONVERSATION_MEMORY_TEXT_MAX_CHARS,
        }),
    );
    Value::Object(source_map)
}

fn copy_memory_source_map_field(
    source: &serde_json::Map<String, Value>,
    target: &mut serde_json::Map<String, Value>,
    key: &str,
) {
    let Some(value) = source.get(key) else {
        return;
    };
    match value {
        Value::Bool(_) | Value::Number(_) => {
            target.insert(key.to_string(), value.clone());
        }
        Value::String(value) => {
            let value = normalize_history_text(value);
            if !value.is_empty() {
                target.insert(
                    key.to_string(),
                    Value::String(truncate_chars(&value, CONVERSATION_MEMORY_SOURCE_MAX_CHARS)),
                );
            }
        }
        _ => {}
    }
}

fn extract_history_constraints(candidates: &[(i64, &'static str, String)]) -> Vec<Value> {
    let markers = [
        "不要",
        "不能",
        "必须",
        "只",
        "不引用",
        "do not",
        "must",
        "only",
    ];
    candidates
        .iter()
        .filter(|(_, role, _)| *role == "User")
        .filter(|(_, _, content)| {
            let lower = content.to_ascii_lowercase();
            markers.iter().any(|marker| lower.contains(marker))
        })
        .rev()
        .take(5)
        .map(|(sequence, _, content)| {
            json!({
                "source": format!("conversation_message:{sequence}"),
                "text": truncate_chars(content, 220),
                "confidence": "explicit",
            })
        })
        .collect()
}

fn memory_text_from_json(memory: &Value) -> String {
    let mut lines = Vec::new();
    if let Some(summary) = memory.get("rollingSummary").and_then(Value::as_str) {
        lines.push(format!("Rolling summary: {summary}"));
    }
    if let Some(facts) = memory.get("facts").and_then(Value::as_array)
        && !facts.is_empty()
    {
        lines.push("Recent user facts:".to_string());
        lines.extend(
            facts
                .iter()
                .take(6)
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .map(|text| format!("- {text}")),
        );
    }
    if let Some(constraints) = memory.get("constraints").and_then(Value::as_array)
        && !constraints.is_empty()
    {
        lines.push("User constraints:".to_string());
        lines.extend(
            constraints
                .iter()
                .take(5)
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .map(|text| format!("- {text}")),
        );
    }
    truncate_chars(&lines.join("\n"), CONVERSATION_MEMORY_TEXT_MAX_CHARS)
}

fn build_older_conversation_summary(candidates: &[(i64, &'static str, String)]) -> String {
    if candidates.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    let start = candidates
        .len()
        .saturating_sub(CONVERSATION_HISTORY_OLDER_SUMMARY_MESSAGES);
    for (_, role, content) in candidates.iter().skip(start) {
        let brief = truncate_chars(content, 180);
        parts.push(format!("{role}: {brief}"));
    }
    truncate_chars(
        &parts.join(" / "),
        CONVERSATION_HISTORY_OLDER_SUMMARY_MAX_CHARS,
    )
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

async fn merge_conversation_file_image_attachments(
    metadata: &mut std::collections::BTreeMap<String, String>,
    user_id: &str,
    context: &WorkspaceContextSummaryResponse,
) -> Result<(), Error> {
    if metadata.remove("attachments.images_json").is_some() {
        tracing::debug!("sse raw image attachment metadata removed: live vision is system-only");
    }

    let image_sources = auto_vision_conversation_file_sources(context);
    tracing::debug!(
        attachment_sources = context.attachment_sources.len(),
        image_sources = image_sources.len(),
        "sse image attachment source scan finished"
    );
    if image_sources.is_empty() {
        return Ok(());
    }

    let preprocess_manager = VisionPreprocessManager::new(get_app_ctx());
    let mut image_summaries = Vec::new();
    for source in image_sources {
        let object_key = source
            .metadata
            .get("objectKey")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Error::Custom(format!(
                    "conversation image attachment missing objectKey for {}",
                    source.ref_key
                ))
            })?;
        let mime_type = source.mime_type.as_deref().ok_or_else(|| {
            Error::Custom(format!(
                "conversation image attachment missing mimeType for {}",
                source.ref_key
            ))
        })?;
        let file_id = Uuid::parse_str(&source.id).map_err(|err| {
            Error::Custom(format!(
                "conversation image attachment invalid file id for {}: {}",
                source.ref_key, err
            ))
        })?;
        if let Some(analysis) = preprocess_manager
            .latest_succeeded_for_file(file_id)
            .await
            .map_err(|err| {
                Error::Custom(format!(
                    "conversation image attachment summary lookup failed: {err:?}"
                ))
            })?
            && let Some(summary) = analysis.summary.as_deref().map(str::trim)
            && !summary.is_empty()
        {
            let agent_context = analysis
                .result_json
                .as_ref()
                .and_then(|value| value.get("agentContext"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            image_summaries.push(AgentVisionImageSummaryAttachment {
                source_ref: source.ref_key.clone(),
                object_key: object_key.to_string(),
                summary: summary.to_string(),
                agent_context,
                analysis_id: analysis.id.to_string(),
                prompt_version: Some(analysis.prompt_version),
                llm_name: Some(analysis.llm_name),
            });
            tracing::debug!(
                ref_key = %source.ref_key,
                object_key,
                "conversation image attachment using preprocessed summary"
            );
            continue;
        }
        tracing::debug!(
            user_id,
            ref_key = %source.ref_key,
            object_key,
            mime_type,
            file_size = ?source.file_size,
            "sse image attachment skipped: preprocessed summary unavailable"
        );
    }

    metadata.insert("attachments.image_count".to_string(), "0".to_string());
    metadata.insert(
        "attachments.image_summary_count".to_string(),
        image_summaries.len().to_string(),
    );
    if !image_summaries.is_empty() {
        metadata.insert(
            "attachments.image_summaries_json".to_string(),
            serde_json::to_string(&image_summaries).map_err(|err| {
                Error::Custom(format!("agent image summary metadata failed: {err}"))
            })?,
        );
        append_context_summary_section(
            metadata,
            "Preprocessed conversation image context:",
            &format_image_summary_context(&image_summaries),
        );
    }
    let image_source = if image_summaries.is_empty() {
        "conversation_file_unavailable"
    } else {
        "conversation_file_preprocessed_summary"
    };
    metadata.insert(
        "attachments.image_source".to_string(),
        image_source.to_string(),
    );
    tracing::debug!(
        image_count = 0,
        image_summary_count = image_summaries.len(),
        image_summaries_json_bytes = metadata
            .get("attachments.image_summaries_json")
            .map_or(0, |value| value.len()),
        "sse image attachment metadata written"
    );
    Ok(())
}

fn format_image_summary_context(image_summaries: &[AgentVisionImageSummaryAttachment]) -> String {
    image_summaries
        .iter()
        .map(|item| {
            let summary = item
                .agent_context
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| item.summary.trim());
            format!(
                "- source_ref: {} | object_key: {} | analysis_id: {} | summary: {}",
                item.source_ref, item.object_key, item.analysis_id, summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_supported_vision_image_mime(value: &str) -> bool {
    is_supported_vision_mime(value)
}

fn auto_vision_conversation_file_sources(
    context: &WorkspaceContextSummaryResponse,
) -> Vec<&WorkspaceContextSourceResponse> {
    context
        .attachment_sources
        .iter()
        .filter(|source| {
            source.source_type == "conversation_file"
                && source.status == "active"
                && source
                    .mime_type
                    .as_deref()
                    .is_some_and(is_supported_vision_image_mime)
        })
        .take(MAX_AUTO_VISION_IMAGES)
        .collect()
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
        "workspaceContextPack.item_count".to_string(),
        context.items.len().to_string(),
    );
    metadata.insert(
        "workspaceContextPack.omitted_json".to_string(),
        serde_json::to_string(&context.omitted)?,
    );
    metadata.insert(
        "workspaceContextPack.omitted_details_json".to_string(),
        serde_json::to_string(&context.omitted_details)?,
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
    append_context_injection_budget(metadata, context)?;
    append_workspace_context_pack_summary(metadata, context);
    Ok(())
}

fn append_context_injection_budget(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &WorkspaceContextPackResponse,
) -> Result<(), serde_json::Error> {
    let readable_refs = context
        .items
        .iter()
        .filter(|item| item.text.is_some())
        .map(|item| item.ref_key.clone())
        .collect::<Vec<_>>();
    let metadata_only_refs = context
        .items
        .iter()
        .filter(|item| item.text.is_none())
        .map(|item| item.ref_key.clone())
        .collect::<Vec<_>>();
    let approximate_chars = context
        .items
        .iter()
        .map(|item| item.text_chars)
        .sum::<usize>();
    let budget = json!({
        "kind": "context_injection_budget",
        "workspacePack": {
            "readiness": context.readiness,
            "itemCount": context.items.len(),
            "readableItemCount": readable_refs.len(),
            "metadataOnlyItemCount": metadata_only_refs.len(),
            "omittedCount": context.omitted_details.len(),
            "approximateChars": approximate_chars,
        },
        "includedRefs": readable_refs,
        "metadataOnlyRefs": metadata_only_refs,
        "omitted": context.omitted_details,
    });
    metadata.insert(
        "context.injection_budget_json".to_string(),
        serde_json::to_string(&budget)?,
    );
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
        .chain(context.campaign_sources.iter())
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
            degraded_reason: Some(
                "No matching local product knowledge was found; external search adapter is not \
                 configured."
                    .to_string(),
            ),
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
            degraded_reason: Some(
                "No matching local product knowledge was found; external search adapter is \
                 configured and search skills may request live sources when the task needs \
                 real-time market data."
                    .to_string(),
            ),
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
            build_conversation_history_summary(&messages, "请直接根据前面补充的信息完成方案", None);

        assert_eq!(summary.message_count, 3);
        assert_eq!(summary.total_message_count, 3);
        assert!(!summary.truncated);
        assert!(summary.text.contains("单价100"));
        assert!(!summary.text.contains("请直接根据前面补充的信息完成方案"));
    }

    #[test]
    fn conversation_history_keeps_recent_window_and_summarizes_older_messages() {
        let mut messages = Vec::new();
        for index in 1 ..= 18 {
            let role = if index % 2 == 0 { "assistant" } else { "user" };
            messages.push(conversation_message(
                index,
                role,
                &format!("第 {index} 轮关于咖啡商品、预算和复盘指标的上下文"),
            ));
        }

        let summary =
            build_conversation_history_summary(&messages, "继续看咖啡预算和复盘指标", None);

        assert_eq!(summary.message_count, 12);
        assert_eq!(summary.total_message_count, 18);
        assert!(summary.truncated);
        assert_eq!(summary.relevant_older_count, 4);
        assert!(summary.older_summary.contains("第 6 轮"));
        assert!(summary.text.contains("Relevant earlier messages:"));
        assert!(summary.text.contains("Earlier summary:"));
        assert!(summary.text.contains("第 18 轮"));
        assert!(summary.text.contains("第 7 轮"));
        assert!(!summary.text.contains("\nUser: 第 5 轮"));
    }

    #[test]
    fn conversation_history_merges_previous_memory() {
        let messages = vec![conversation_message(
            1,
            "user",
            "本轮补充：预算不能超过 3000",
        )];
        let previous_memory = json!({
            "kind": "conversation_memory_summary",
            "rollingSummary": "用户之前确认目标是提升夏季防晒商品转化。",
            "facts": [
                {
                    "source": "conversation_message:old",
                    "text": "用户主推轻薄防晒衣",
                    "confidence": "explicit"
                }
            ],
            "decisions": [],
            "constraints": [],
            "openQuestions": ["是否需要按渠道拆分预算"]
        });

        let summary = build_conversation_history_summary(&messages, "继续", Some(&previous_memory));

        assert!(summary.memory_text.contains("轻薄防晒衣"));
        assert!(
            summary
                .memory_json
                .get("rollingSummary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("夏季防晒商品")
        );
        assert_eq!(
            summary.memory_json["sourceMap"]["previousMemoryAvailable"],
            true
        );
        assert!(
            summary
                .memory_json
                .get("constraints")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .any(|item| item["text"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("不能超过"))
        );
    }

    #[test]
    fn conversation_history_compacts_previous_memory_budget() {
        let long_fact = format!("用户主推轻薄防晒衣 {}", "长描述".repeat(180));
        let long_summary = "旧摘要 ".repeat(600);
        let previous_memory = json!({
            "kind": "conversation_memory_summary",
            "rollingSummary": long_summary,
            "facts": [
                {
                    "source": "conversation_message:old",
                    "text": long_fact,
                    "confidence": "explicit"
                },
                {
                    "source": "conversation_message:duplicate",
                    "text": " 用户主推轻薄防晒衣 ",
                    "confidence": "explicit"
                }
            ],
            "decisions": [
                {"source": "old", "text": "保留价格约束", "confidence": "generated"},
                {"source": "old-2", "text": "保留价格约束", "confidence": "generated"}
            ],
            "constraints": [
                {"source": "old", "text": "预算不能超过 3000", "confidence": "explicit"}
            ],
            "openQuestions": [
                "是否需要按渠道拆分预算",
                "是否需要按渠道拆分预算",
                {"source": "old", "text": "是否继续做直播脚本", "confidence": "open"}
            ],
            "sourceMap": {
                "previousMemoryAvailable": true,
                "lastSessionId": "session-with-extra-long-suffix".repeat(20),
                "ignoredNested": {"x": 1}
            }
        });

        let messages = vec![
            conversation_message(1, "user", "本轮补充：预算不能超过 3000"),
            conversation_message(2, "user", "用户主推轻薄防晒衣"),
        ];
        let summary = build_conversation_history_summary(&messages, "继续", Some(&previous_memory));
        let memory = summary.memory_json;
        let facts = memory["facts"].as_array().unwrap();
        let decisions = memory["decisions"].as_array().unwrap();
        let open_questions = memory["openQuestions"].as_array().unwrap();

        assert!(
            memory["rollingSummary"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
                <= CONVERSATION_MEMORY_ROLLING_SUMMARY_MAX_CHARS + 3
        );
        assert!(facts.len() <= CONVERSATION_MEMORY_FACTS_MAX_ITEMS);
        assert_eq!(
            facts
                .iter()
                .filter(|item| item["text"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("用户主推轻薄防晒衣"))
                .count(),
            1
        );
        assert_eq!(
            decisions
                .iter()
                .filter(|item| item["text"].as_str() == Some("保留价格约束"))
                .count(),
            1
        );
        assert_eq!(open_questions.len(), 2);
        assert_eq!(
            memory["sourceMap"]["memoryBudget"]["factsMaxItems"],
            CONVERSATION_MEMORY_FACTS_MAX_ITEMS
        );
        assert!(memory["sourceMap"].get("ignoredNested").is_none());
    }

    #[test]
    fn completed_conversation_memory_adds_last_turn_output() {
        let link_state = AgentWorkspaceLinkState {
            user_id: "user-1".to_string(),
            session_id: "session-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            run_id: "run-1".to_string(),
            latest_output: Some("建议把主卖点压缩成三条，并保留价格约束。".to_string()),
            completed: true,
            conversation_id: Some("conversation-1".to_string()),
            input: "继续完善方案".to_string(),
            conversation_memory_summary: Some(json!({
                "kind": "conversation_memory_summary",
                "rollingSummary": "已有商品定位。",
                "facts": [],
                "decisions": [],
                "constraints": [],
                "openQuestions": [],
                "sourceMap": {}
            })),
        };

        let memory = build_completed_conversation_memory(&link_state).unwrap();

        assert_eq!(memory["lastUserInput"], "继续完善方案");
        assert!(
            memory["lastAssistantOutput"]
                .as_str()
                .unwrap_or_default()
                .contains("主卖点")
        );
        assert_eq!(memory["sourceMap"]["lastSessionId"], "session-1");
        assert_eq!(memory["sourceMap"]["persisted"], true);
    }

    #[test]
    fn completed_conversation_memory_clamps_last_turn_output() {
        let long_output = format!("建议保留利润率判断。{}", "后续执行细节".repeat(200));
        let link_state = AgentWorkspaceLinkState {
            user_id: "user-1".to_string(),
            session_id: "session-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            run_id: "run-1".to_string(),
            latest_output: Some(long_output),
            completed: true,
            conversation_id: Some("conversation-1".to_string()),
            input: "请继续完善利润率方案。".repeat(80),
            conversation_memory_summary: Some(json!({
                "kind": "conversation_memory_summary",
                "rollingSummary": "已有商品定位。",
                "facts": [],
                "decisions": [],
                "constraints": [],
                "openQuestions": [],
                "sourceMap": {}
            })),
        };

        let memory = build_completed_conversation_memory(&link_state).unwrap();

        assert!(
            memory["lastUserInput"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
                <= CONVERSATION_MEMORY_LAST_USER_INPUT_MAX_CHARS + 3
        );
        assert!(
            memory["lastAssistantOutput"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
                <= CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS + 3
        );
        assert_eq!(
            memory["sourceMap"]["memoryBudget"]["lastAssistantOutputMaxChars"],
            CONVERSATION_MEMORY_LAST_ASSISTANT_OUTPUT_MAX_CHARS
        );
        assert!(
            memory["decisions"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
                <= CONVERSATION_MEMORY_ITEM_MAX_CHARS + 3
        );
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
            None,
        );

        assert_eq!(metadata["conversation.conversation_id"], "conversation-1");
        assert_eq!(metadata["conversation.history_message_count"], "3");
        assert_eq!(metadata["conversation.history_total_message_count"], "3");
        assert_eq!(metadata["conversation.history_truncated"], "false");
        assert_eq!(metadata["conversation.history_relevant_older_count"], "0");
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
                campaigns: 1,
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
            campaign_sources: vec![WorkspaceContextSourceResponse {
                source_type: "campaign".to_string(),
                id: "campaign-1".to_string(),
                title: Some("Launch campaign".to_string()),
                mime_type: Some("application/json".to_string()),
                file_size: None,
                status: "active".to_string(),
                ref_key: "campaign:campaign-1".to_string(),
                preview: Some("Launch campaign".to_string()),
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
                    "campaign:campaign-1",
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
            "product:product-1,conversation_file:file-1,campaign:campaign-1,product_asset:asset-1,\
             workspace_artifact:artifact-1"
        );
        assert!(metadata["context.summary"].contains("Selected product."));
        assert!(
            metadata["context.combined_summary"]
                .contains("Workspace file/material/artifact context:")
        );
        assert!(metadata["workspaceContext.sources_json"].contains("conversation_file"));
    }

    #[test]
    fn supported_vision_image_mime_allows_common_web_images_only() {
        assert!(is_supported_vision_image_mime("image/png"));
        assert!(is_supported_vision_image_mime("IMAGE/JPEG"));
        assert!(is_supported_vision_image_mime("image/webp"));
        assert!(!is_supported_vision_image_mime("image/gif"));
        assert!(!is_supported_vision_image_mime("application/pdf"));
    }

    #[test]
    fn auto_vision_sources_use_active_conversation_images_with_limit() {
        let context = WorkspaceContextSummaryResponse {
            workspace_id: "workspace-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            product_id: None,
            run_id: None,
            readiness: "ready".to_string(),
            summary: "Workspace context is ready.".to_string(),
            counts: service::dto::workspace::WorkspaceContextSummaryCounts {
                campaigns: 0,
                conversation_files: 7,
                product_assets: 0,
                product_materials: 0,
                workspace_artifacts: 0,
            },
            attachment_sources: vec![
                workspace_context_source("conversation_file", "file-1", "image/png", "active"),
                workspace_context_source("conversation_file", "file-2", "image/jpeg", "active"),
                workspace_context_source(
                    "conversation_file",
                    "file-3",
                    "application/pdf",
                    "active",
                ),
                workspace_context_source("conversation_file", "file-4", "image/webp", "deleted"),
                workspace_context_source("product_asset", "asset-1", "image/png", "active"),
                workspace_context_source("conversation_file", "file-5", "image/webp", "active"),
                workspace_context_source("conversation_file", "file-6", "IMAGE/JPEG", "active"),
                workspace_context_source("conversation_file", "file-7", "image/png", "active"),
            ],
            campaign_sources: Vec::new(),
            product_asset_sources: Vec::new(),
            product_material_sources: Vec::new(),
            artifact_sources: Vec::new(),
            missing: Vec::new(),
            warnings: Vec::new(),
            agent_context: json!({}),
        };

        let ids = auto_vision_conversation_file_sources(&context)
            .iter()
            .map(|source| source.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["file-1", "file-2", "file-5", "file-6"]);
    }

    #[test]
    fn image_summary_context_appends_to_combined_summary() {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert(
            "context.combined_summary".to_string(),
            "Existing context.".to_string(),
        );
        let image_summaries = vec![AgentVisionImageSummaryAttachment {
            source_ref: "conversation_file:file-1".to_string(),
            object_key: "images/file-1.png".to_string(),
            summary: "图片是一张冷萃咖啡促销海报。".to_string(),
            agent_context: Some("海报主标题是低糖冷萃，包含买二送一促销。".to_string()),
            analysis_id: "analysis-1".to_string(),
            prompt_version: Some("1".to_string()),
            llm_name: Some("vision-llm".to_string()),
        }];

        append_context_summary_section(
            &mut metadata,
            "Preprocessed conversation image context:",
            &format_image_summary_context(&image_summaries),
        );

        let combined = metadata
            .get("context.combined_summary")
            .expect("combined summary should exist");
        assert!(combined.contains("Existing context."));
        assert!(combined.contains("Preprocessed conversation image context:"));
        assert!(combined.contains("conversation_file:file-1"));
        assert!(combined.contains("海报主标题是低糖冷萃"));
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
                campaigns: 0,
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
            omitted_details: Vec::new(),
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

    fn workspace_context_source(
        source_type: &str,
        id: &str,
        mime_type: &str,
        status: &str,
    ) -> WorkspaceContextSourceResponse {
        WorkspaceContextSourceResponse {
            source_type: source_type.to_string(),
            id: id.to_string(),
            title: Some(id.to_string()),
            mime_type: Some(mime_type.to_string()),
            file_size: Some(1024),
            status: status.to_string(),
            ref_key: format!("{source_type}:{id}"),
            preview: Some(id.to_string()),
            metadata: json!({}),
        }
    }
}
