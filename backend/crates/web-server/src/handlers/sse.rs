use std::{
    convert::Infallible,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use agent::api::{FormDraftInput, FormDraftResult, SessionEvent, WorkerReportStatus};
use axum::{
    Extension, Json,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use db_core::error::BizResult;
use futures_util::Stream;
use serde::Serialize;
use serde_json::{Value, json};
use service::{
    api::{
        agent_context::AgentContextApi, app::AppApi, approval_audit::ApprovalAuditApi,
        briefing_ops::BriefingOpsApi, conversation::ConversationApi, team_usage::TeamUsageApi,
        vision_preprocess_manager::VisionPreprocessManager, workspace::WorkspaceApi,
    },
    dto::{
        agent_context::{
            AgentContextSource, AgentRetrievalContextResponse, BuildAgentProductContextRequest,
            BuildAgentRetrievalContextRequest,
        },
        approval_audit::{CreateAuditLogRequest, CreateToolApprovalRequest, ToolApprovalResponse},
        briefing_ops::{AlertResponse, CreateAlertRequest},
        conversation::ConversationMessageResponse,
        team_usage::{
            AgentQuestionOutputStatus, AgentQuestionPreflight, AgentQuestionPreflightRequest,
            RecordAgentQuestionOutputRequest, RecordAgentQuestionStartedRequest,
        },
        workspace::{
            AgentLearningRecordResponse, AgentQualityEvaluationResponse, AgentWorkspaceTaskNode,
            BuildAgentLearningContextRequest, CompleteAgentWorkspaceRunRequest,
            EvaluateAgentWorkspaceRunRequest, FailAgentWorkspaceRunRequest,
            LinkAgentWorkspaceRunRequest, LinkAgentWorkspaceTasksRequest,
            MarkAgentWorkspaceTaskStartedRequest, MaterializeAgentCampaignDraftRequest,
            MaterializeAgentContentArtifactRequest, MaterializeAgentWorkspaceTaskRequest,
            RecordAgentSearchEvidenceRequest, RecordAgentSkillRequestedRequest,
            RecordAgentSkillResultRequest, RecordAgentWorkspaceTaskEvaluationRequest,
            RecordAgentWorkspaceTaskReportRequest, WorkspaceArtifactResponse,
            WorkspaceContextPackResponse, WorkspaceContextSourceResponse,
            WorkspaceContextSummaryResponse, WorkspaceRunResponse, WorkspaceRunTaskResponse,
        },
    },
};
use toolcraft_axum_kit::{ApiError, CommonError, middleware::auth_mw::AuthUser};
use utils::image::is_supported_vision_mime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::sse::{AgentStreamContextSelection, AgentStreamRequest, FormDraftStreamRequest},
    error::{Error, from_biz_error},
    handlers::app::form_draft::{
        FORM_DRAFT_AGENT_FAILURE_MESSAGE, FormDraftGenerationMode, campaign_form_draft_input,
        map_agent_campaign_form_draft, map_agent_product_form_draft,
        parse_form_draft_generation_mode, product_form_draft_input,
    },
    statics::{
        agent_manager::{run_agent_stream, run_form_draft},
        db_manager::get_app_ctx,
        external_search::get_external_search_config,
    },
};

const MAX_AUTO_VISION_IMAGES: usize = 4;
const AGENT_STREAM_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

fn should_include_implicit_conversation_files(
    input: &str,
    context: &AgentStreamContextSelection,
) -> bool {
    if has_explicit_turn_context(context) || input_requests_independent_context(input) {
        return false;
    }
    input_requests_previous_conversation_files(input)
}

fn should_include_conversation_history_grounding(
    input: &str,
    context: &AgentStreamContextSelection,
) -> bool {
    if input_requests_independent_context(input) {
        return false;
    }
    has_explicit_turn_context(context) || input_requests_previous_conversation_context(input)
}

fn effective_context_selection_for_input(
    input: &str,
    mut context: AgentStreamContextSelection,
) -> (AgentStreamContextSelection, Option<&'static str>) {
    if !has_explicit_turn_context(&context)
        || !input_requests_generic_category_answer(input)
        || input_requests_selected_turn_context(input)
    {
        return (context, None);
    }

    context.product_ids.clear();
    context.campaign_ids.clear();
    context.context_refs.clear();
    (
        context,
        Some("generic_category_question_without_selected_context_request"),
    )
}

fn has_explicit_turn_context(context: &AgentStreamContextSelection) -> bool {
    !context.product_ids.is_empty()
        || !context.campaign_ids.is_empty()
        || !context.context_refs.is_empty()
        || context.first_product_ref_id().is_some()
        || !context.campaign_context_refs().is_empty()
}

fn merge_agent_stream_context_selection_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: &AgentStreamContextSelection,
) {
    if !context.product_ids.is_empty() {
        metadata.insert(
            "context.product_ids".to_string(),
            context.product_ids.join(","),
        );
    }
    if !context.campaign_ids.is_empty() {
        metadata.insert(
            "context.campaign_ids".to_string(),
            context.campaign_ids.join(","),
        );
    }
    if !context.context_refs.is_empty() {
        append_context_refs(metadata, &context.context_refs);
    }
    let campaign_refs = context.campaign_context_refs();
    if !campaign_refs.is_empty() {
        append_context_refs(metadata, &campaign_refs);
    }
    let product_refs = context.product_context_refs();
    if !product_refs.is_empty() {
        append_context_refs(metadata, &product_refs);
    }
}

fn input_requests_independent_context(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    [
        "新话题",
        "换个问题",
        "独立问题",
        "独立泛问",
        "不引用",
        "不要引用",
        "不沿用",
        "不要沿用",
        "不看前面",
        "不要看前面",
        "别看前面",
        "no reference",
        "without reference",
        "do not reference",
        "ignore previous",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn input_requests_generic_category_answer(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let direct_markers = [
        "各类商品",
        "各种商品",
        "多类商品",
        "所有商品",
        "全品类",
        "多品类",
        "类目级",
        "品类级",
        "商品通用",
        "产品通用",
        "商品类",
        "类商品",
        "不指定商品",
        "不绑定商品",
        "不选择商品",
        "all products",
        "product categories",
        "generic product",
    ];
    if direct_markers.iter().any(|marker| lower.contains(marker)) {
        return true;
    }

    lower.contains("通用")
        && [
            "框架",
            "判断",
            "方法",
            "流程",
            "模型",
            "运营",
            "商品",
            "产品",
            "电商",
            "framework",
            "method",
            "product",
            "commerce",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn input_requests_selected_turn_context(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let selected_context_markers = [
        "这个商品",
        "该商品",
        "这款商品",
        "选中商品",
        "所选商品",
        "引用商品",
        "引用的商品",
        "当前商品",
        "这个产品",
        "该产品",
        "这款产品",
        "选中产品",
        "所选产品",
        "引用产品",
        "引用的产品",
        "当前产品",
        "这个活动",
        "该活动",
        "这场活动",
        "选中活动",
        "引用活动",
        "引用的活动",
        "这份资料",
        "这个资料",
        "这份文件",
        "这个文件",
        "这份附件",
        "这个附件",
        "这份表格",
        "这个表格",
        "这张图",
        "这张图片",
        "这幅图",
        "图里",
        "截图",
        "上传的",
        "根据这份",
        "结合这份",
        "基于这份",
        "根据这个商品",
        "结合这个商品",
        "基于这个商品",
        "根据选中",
        "结合选中",
        "基于选中",
        "selected product",
        "this product",
        "current product",
        "selected file",
        "this file",
        "uploaded file",
        "this image",
        "attached file",
    ];
    selected_context_markers
        .iter()
        .any(|marker| lower.contains(marker))
}

fn input_requests_previous_conversation_files(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let material_markers = [
        "文件",
        "资料",
        "附件",
        "图片",
        "截图",
        "图里",
        "这张图",
        "上传",
        "表格",
        "csv",
        "excel",
        "pdf",
        "docx",
        "file",
        "image",
        "attachment",
        "spreadsheet",
    ];
    let followup_markers = [
        "刚才",
        "刚刚",
        "前面",
        "上面",
        "之前",
        "上一轮",
        "这个",
        "这份",
        "这张",
        "继续",
        "还是",
        "根据",
        "按",
        "里面",
        "里头",
        "uploaded",
        "previous",
        "above",
        "earlier",
        "this",
    ];
    material_markers.iter().any(|marker| lower.contains(marker))
        && followup_markers.iter().any(|marker| lower.contains(marker))
}

fn input_requests_previous_conversation_context(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    [
        "继续",
        "接着",
        "沿用",
        "根据前面",
        "按前面",
        "前面",
        "刚才",
        "刚刚",
        "上面",
        "之前",
        "上一轮",
        "同一个",
        "这个商品",
        "这个活动",
        "这款商品",
        "这些",
        "再给",
        "再写",
        "补充",
        "follow up",
        "same",
        "previous",
        "above",
        "earlier",
        "continue",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

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

pub async fn stream_form_draft(
    Extension(_auth_user): Extension<AuthUser>,
    Json(req): Json<FormDraftStreamRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    req.validate().map_err(Error::from)?;
    let target = form_draft_stream_target(req).map_err(bad_request)?;
    let kind = target.kind;
    let form_type = kind.form_type().to_string();
    let input = target.input;

    let stream = async_stream::stream! {
        yield Ok(typed_camel_json_event(
            "form_draft_started",
            form_draft_started_payload(&form_type),
        ));
        yield Ok(typed_camel_json_event(
            "form_draft_input_preflighted",
            form_draft_input_preflighted_payload(&form_type, &input),
        ));
        yield Ok(typed_camel_json_event(
            "form_draft_agent_started",
            form_draft_agent_started_payload(&form_type),
        ));
        yield Ok(typed_camel_json_event(
            "form_draft_output_validation_started",
            form_draft_output_validation_started_payload(&form_type),
        ));

        match run_form_draft(input).await {
            Ok(result) => {
                yield Ok(typed_camel_json_event(
                    "form_draft_output_validation_passed",
                    form_draft_output_validation_passed_payload(&form_type),
                ));
                match form_draft_response_value(kind, result) {
                    Ok(response) => {
                        for (event, payload) in form_draft_mapped_success_events(&form_type, response) {
                            yield Ok(typed_camel_json_event(event, payload));
                        }
                        yield Ok(json_event("done", json!({})));
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "form draft response serialization failed");
                        yield Ok(typed_camel_json_event(
                            "form_draft_failed",
                            form_draft_failed_payload(&form_type, "response_mapping_failed"),
                        ));
                        yield Ok(json_event(
                            "error",
                            json!({ "message": FORM_DRAFT_AGENT_FAILURE_MESSAGE }),
                        ));
                        yield Ok(json_event("done", json!({})));
                    }
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "form draft stream failed before form fill");
                yield Ok(typed_camel_json_event(
                    "form_draft_failed",
                    form_draft_failed_payload(&form_type, "agent_run_failed"),
                ));
                yield Ok(json_event(
                    "error",
                    json!({ "message": FORM_DRAFT_AGENT_FAILURE_MESSAGE }),
                ));
                yield Ok(json_event("done", json!({})));
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
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
    let (mut metadata, context_selection) = req.metadata.into_parts().map_err(bad_request)?;
    let (context_selection, ignored_context_reason) =
        effective_context_selection_for_input(&input, context_selection);
    if let Some(reason) = ignored_context_reason {
        metadata.insert(
            "context.explicitSelectionIgnored".to_string(),
            reason.to_string(),
        );
    }
    merge_agent_stream_context_selection_metadata(&mut metadata, &context_selection);
    let conversation_id = context_selection.conversation_id.clone();
    let mut conversation_memory_summary: Option<Value> = None;
    let mut conversation_context_loaded: Option<Value> = None;
    let selected_product_id = context_selection
        .product_ids
        .first()
        .cloned()
        .or_else(|| context_selection.first_product_ref_id());
    let mut workspace_context_refs = context_selection.context_refs.clone();
    let include_implicit_conversation_files =
        should_include_implicit_conversation_files(&input, &context_selection);
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
    let team_usage_api = TeamUsageApi::new(get_app_ctx());
    let team_usage_preflight = team_usage_api
        .preflight_agent_question(AgentQuestionPreflightRequest {
            user_id: user_id.clone(),
            conversation_id: conversation_id.clone(),
            session_id: session_id.clone(),
            agent_id: agent_id.clone(),
        })
        .await
        .map_err(from_biz_error)?;
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
        let include_conversation_history_grounding =
            should_include_conversation_history_grounding(&input, &context_selection);
        let conversation_history_summary = merge_conversation_history_metadata(
            &mut metadata,
            conversation_id,
            &conversation.messages,
            &input,
            previous_memory.as_ref(),
            include_conversation_history_grounding,
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
    let external_search_config = get_external_search_config();
    let external_search_health = external_search_config.health_status();
    let external_search_configured = external_search_health.configured;
    let external_search_available = external_search_health.usable;
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
            external_search_configured,
            external_search_status: Some(external_search_health.status.clone()),
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
    team_usage_api
        .record_agent_question_started(RecordAgentQuestionStartedRequest {
            preflight: team_usage_preflight.clone(),
            workspace_id: Some(linked_workspace.workspace.id.clone()),
            run_id: Some(linked_workspace.run.id.clone()),
            input: input.clone(),
        })
        .await
        .map_err(from_biz_error)?;
    metadata.insert("workspace.source".to_string(), "agent_session".to_string());
    metadata.insert(
        "workspaceContext.includeImplicitConversationFiles".to_string(),
        include_implicit_conversation_files.to_string(),
    );
    let active_run_id = Some(linked_workspace.run.id.clone());
    let workspace_context = workspace_api
        .context_summary(service::dto::workspace::GetWorkspaceContextSummaryRequest {
            user_id: user_id.clone(),
            workspace_id: linked_workspace.workspace.id.clone(),
            conversation_id: conversation_id.clone(),
            product_id: selected_product_id,
            include_workspace_product_context: false,
            include_implicit_conversation_files,
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
            include_implicit_conversation_files,
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
        team_usage_preflight,
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
            workspace_context_loaded_public_payload(&workspace_context),
        ));
        yield Ok(typed_camel_json_event(
            "workspace_context_pack_loaded",
            workspace_context_pack_loaded_public_payload(&workspace_context_pack),
        ));
        yield Ok(typed_camel_json_event(
            "rag_context_loaded",
            rag_context_loaded_public_payload(&retrieval_context),
        ));
        let mut runtime_started = false;
        loop {
            let event = match tokio::time::timeout(AGENT_STREAM_STARTUP_TIMEOUT, events.recv()).await {
                Ok(Some(event)) => event,
                Ok(None) => break,
                Err(_) if !runtime_started => {
                    let reason = agent_stream_startup_timeout_message(AGENT_STREAM_STARTUP_TIMEOUT);
                    if !link_state.completed {
                        let response = workspace_api
                            .fail_agent_run(FailAgentWorkspaceRunRequest {
                                user_id: link_state.user_id.clone(),
                                workspace_id: link_state.workspace_id.clone(),
                                run_id: link_state.run_id.clone(),
                                reason: "agent stream startup timed out".to_string(),
                            })
                            .await;
                        match response {
                            Ok(value) => {
                                link_state.completed = true;
                                yield Ok(typed_camel_json_event(
                                    "workspace_run_failed",
                                    workspace_run_public_payload(&value),
                                ));
                            }
                            Err(err) => yield Ok(persistence_error_event("fail_agent_run", err)),
                        }
                    }
                    yield Ok(json_event("output", json!({ "content": reason })));
                    yield Ok(json_event("done", json!({})));
                    return;
                }
                Err(_) => {
                    continue;
                }
            };
            let event_name = event.event_name();
            let event_value = agent_session_event_public_payload(&event);
            yield Ok(json_event(event_name, event_value));

            match &event {
                SessionEvent::MemoryLoaded { .. }
                | SessionEvent::ContextLoaded { .. }
                | SessionEvent::CommanderThinking { .. }
                | SessionEvent::RouteDecided { .. }
                | SessionEvent::DispatchPlanned { .. } => runtime_started = true,
                SessionEvent::TaskGraphPlanned { graph, .. } => {
                    runtime_started = true;
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
                            workspace_run_task_updated_public_payload(&value),
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
                            status: if matches!(
                                report.status,
                                WorkerReportStatus::Failed | WorkerReportStatus::TimedOut
                            ) {
                                "failed".to_string()
                            } else {
                                "completed".to_string()
                            },
                        })
                        .await;
                    match response {
                        Ok(value) => yield Ok(typed_camel_json_event(
                            "workspace_run_task_updated",
                            workspace_run_task_updated_public_payload(&value),
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
                            workspace_run_task_updated_public_payload(&value),
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
                            agent_learning_record_created_public_payload(&record),
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
                                    workspace_artifact_created_public_payload(&artifact),
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
                            workspace_artifact_created_public_payload(&artifact),
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
                                workspace_artifact_created_public_payload(&artifact),
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
                                    approval_required_public_payload(
                                        &approval,
                                        alert.as_ref(),
                                        ApprovalRequiredPublicPayloadRef {
                                            skill_id,
                                            request_id,
                                            role,
                                            risk_level: &risk_level,
                                            reason: &reason,
                                        },
                                    ),
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
                }
                SessionEvent::Failed { reason, .. } => {
                    if let Err(err) = record_agent_question_output_usage(
                        &link_state,
                        Some(reason.clone()),
                        AgentQuestionOutputStatus::Failed,
                    )
                    .await
                    {
                        yield Ok(persistence_error_event("record_agent_question_output", err));
                    }
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
                                    workspace_run_public_payload(&value),
                                ));
                            }
                            Err(err) => yield Ok(persistence_error_event("fail_agent_run", err)),
                        }
                    }
                    yield Ok(json_event("error", json!({ "message": reason })));
                    return;
                }
                SessionEvent::Finished { .. } => {
                    if let Err(err) = record_agent_question_output_usage(
                        &link_state,
                        link_state.latest_output.clone(),
                        AgentQuestionOutputStatus::Completed,
                    )
                    .await
                    {
                        yield Ok(persistence_error_event("record_agent_question_output", err));
                    }
                    yield Ok(json_event(
                        "finished",
                        json!({
                            "type": "finished",
                            "session_id": link_state.session_id,
                        }),
                    ));
                    if !link_state.completed {
                        let completion = finalize_completed_agent_run(link_state.clone()).await;
                        if let Some(run) = completion.run {
                            yield Ok(typed_camel_json_event(
                                "workspace_run_completed",
                                run,
                            ));
                        }
                        if let Some(artifact) = completion.artifact {
                            yield Ok(typed_camel_json_event(
                                "workspace_artifact_created",
                                artifact,
                            ));
                        }
                        if let Some(quality) = completion.quality {
                            yield Ok(typed_camel_json_event("quality_checked", quality));
                        }
                        if let Some(memory) = completion.memory {
                            yield Ok(typed_camel_json_event(
                                "conversation_memory_updated",
                                memory,
                            ));
                        }
                        for error in completion.persistence_errors {
                            yield Ok(typed_json_event("workspace_run_persistence_error", error));
                        }
                        link_state.completed = true;
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
    team_usage_preflight: AgentQuestionPreflight,
}

async fn record_agent_question_output_usage(
    link_state: &AgentWorkspaceLinkState,
    output: Option<String>,
    status: AgentQuestionOutputStatus,
) -> BizResult<()> {
    TeamUsageApi::new(get_app_ctx())
        .record_agent_question_output(RecordAgentQuestionOutputRequest {
            preflight: link_state.team_usage_preflight.clone(),
            workspace_id: Some(link_state.workspace_id.clone()),
            run_id: Some(link_state.run_id.clone()),
            output,
            status,
        })
        .await
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

fn agent_stream_startup_timeout_message(timeout: Duration) -> String {
    let seconds = timeout.as_secs().max(1);
    [
        "## 当前状态".to_string(),
        String::new(),
        format!("当前 AI 会话在 {seconds}s 内没有成功进入执行阶段，本轮先结束等待。"),
        String::new(),
        "## 对本轮回答的影响".to_string(),
        String::new(),
        "- 已收到你的问题和本轮选择的资料。".to_string(),
        "- 本轮没有开始可靠的智能体协作，所以不能把它当作业务判断结果。".to_string(),
        "- 这通常是模型服务繁忙或上一个会话仍在排队导致的运行问题。".to_string(),
        String::new(),
        "## 下一步".to_string(),
        String::new(),
        "- 可以稍后重试同一个问题。".to_string(),
        "- 如果问题很急，可以先减少资料或协作角色，或切到快速回答继续问。".to_string(),
    ]
    .join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormDraftStreamKind {
    Product,
    Campaign,
}

#[derive(Debug)]
struct FormDraftStreamTarget {
    kind: FormDraftStreamKind,
    input: FormDraftInput,
}

impl FormDraftStreamKind {
    fn form_type(self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Campaign => "campaign",
        }
    }
}

fn form_draft_stream_target(req: FormDraftStreamRequest) -> Result<FormDraftStreamTarget, String> {
    match req.form_type.trim() {
        "product" => {
            if req.campaign.is_some() {
                return Err("formType=product must not include campaign payload".to_string());
            }
            let product = req
                .product
                .ok_or_else(|| "formType=product requires product payload".to_string())?;
            product.validate().map_err(|err| err.to_string())?;
            validate_agent_generation_mode(product.generation_mode.as_deref())?;
            Ok(FormDraftStreamTarget {
                kind: FormDraftStreamKind::Product,
                input: product_form_draft_input(&product),
            })
        }
        "campaign" => {
            if req.product.is_some() {
                return Err("formType=campaign must not include product payload".to_string());
            }
            let campaign = req
                .campaign
                .ok_or_else(|| "formType=campaign requires campaign payload".to_string())?;
            campaign.validate().map_err(|err| err.to_string())?;
            validate_agent_generation_mode(campaign.generation_mode.as_deref())?;
            Ok(FormDraftStreamTarget {
                kind: FormDraftStreamKind::Campaign,
                input: campaign_form_draft_input(&campaign),
            })
        }
        _ => Err("formType must be product or campaign".to_string()),
    }
}

fn validate_agent_generation_mode(value: Option<&str>) -> Result<(), String> {
    match parse_form_draft_generation_mode(value)? {
        FormDraftGenerationMode::AgentLlm => Ok(()),
        FormDraftGenerationMode::TemplateSmartFill => {
            Err("form draft SSE requires generationMode=agent_llm".to_string())
        }
    }
}

fn form_draft_response_value(
    kind: FormDraftStreamKind,
    result: FormDraftResult,
) -> Result<Value, serde_json::Error> {
    match kind {
        FormDraftStreamKind::Product => serde_json::to_value(map_agent_product_form_draft(result)),
        FormDraftStreamKind::Campaign => {
            serde_json::to_value(map_agent_campaign_form_draft(result))
        }
    }
}

fn form_draft_started_payload(form_type: &str) -> Value {
    json!({
        "formType": form_type,
        "generationMode": "agent_llm",
        "directWrite": false,
        "requiresUserReview": true,
    })
}

fn form_draft_input_preflighted_payload(form_type: &str, input: &FormDraftInput) -> Value {
    let context_summary_available = input
        .context_summary
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let existing_field_count = count_existing_form_values(&input.existing_values);

    json!({
        "formType": form_type,
        "generationMode": "agent_llm",
        "directWrite": false,
        "requiresUserReview": true,
        "inputSummary": {
            "existingFieldCount": existing_field_count,
            "userInputAvailable": existing_field_count > 0 || context_summary_available,
            "contextSummaryAvailable": context_summary_available
        }
    })
}

fn count_existing_form_values(value: &Value) -> usize {
    match value {
        Value::Object(object) => object
            .values()
            .filter(|value| form_draft_value_present(value))
            .count(),
        value => usize::from(form_draft_value_present(value)),
    }
}

fn form_draft_value_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(_) | Value::Number(_) => true,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(items) => items.iter().any(form_draft_value_present),
        Value::Object(object) => object.values().any(form_draft_value_present),
    }
}

fn form_draft_agent_started_payload(form_type: &str) -> Value {
    json!({
        "formType": form_type,
        "systemAgentId": "form_draft_generator",
        "route": "web_server_agent_manager",
        "llm": {
            "used": true,
            "route": "web_server_agent_manager"
        }
    })
}

fn form_draft_output_validation_started_payload(form_type: &str) -> Value {
    json!({
        "formType": form_type,
        "draftQuality": {
            "invalidModelJsonPolicy": "reject_before_form_fill"
        }
    })
}

fn form_draft_output_validation_passed_payload(form_type: &str) -> Value {
    json!({
        "formType": form_type,
        "draftQuality": {
            "invalidModelJsonPolicy": "reject_before_form_fill",
            "validated": true
        }
    })
}

fn form_draft_review_ready_payload(form_type: &str) -> Value {
    json!({
        "formType": form_type,
        "generationMode": "agent_llm",
        "directWrite": false,
        "requiresUserReview": true,
        "saveBoundary": {
            "directWrite": false,
            "requiresUserReview": true
        }
    })
}

fn form_draft_completed_payload(form_type: &str, response: Value) -> Value {
    json!({
        "formType": form_type,
        "generationMode": "agent_llm",
        "directWrite": false,
        "requiresUserReview": true,
        "response": response,
    })
}

fn form_draft_mapped_success_events(
    form_type: &str,
    response: Value,
) -> Vec<(&'static str, Value)> {
    vec![
        (
            "form_draft_review_ready",
            form_draft_review_ready_payload(form_type),
        ),
        (
            "form_draft_completed",
            form_draft_completed_payload(form_type, response),
        ),
    ]
}

fn form_draft_failed_payload(form_type: &str, failure_kind: &'static str) -> Value {
    json!({
        "formType": form_type,
        "generationMode": "agent_llm",
        "failureKind": failure_kind,
        "messageAvailable": true,
        "directWrite": false,
        "requiresUserReview": true,
        "draftQuality": {
            "invalidModelJsonPolicy": "reject_before_form_fill",
            "validated": false
        },
        "saveBoundary": {
            "directWrite": false,
            "requiresUserReview": true
        }
    })
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
        persistence_error_payload(action, &err),
    )
}

fn persistence_error_payload(action: &'static str, err: &db_core::error::BizError) -> Value {
    json!({
        "action": action,
        "code": err.code(),
        "message": err.message(),
    })
}

fn log_post_stream_persistence_error(
    action: &'static str,
    err: &db_core::error::BizError,
    link_state: &AgentWorkspaceLinkState,
) {
    tracing::warn!(
        action = action,
        code = %err.code(),
        message = %err.message(),
        run_id = %link_state.run_id,
        session_id = %link_state.session_id,
        "agent sse post-stream persistence failed"
    );
}

fn typed_persistence_error_payload(
    action: &'static str,
    err: db_core::error::BizError,
    link_state: &AgentWorkspaceLinkState,
) -> Value {
    log_post_stream_persistence_error(action, &err, link_state);
    persistence_error_payload(action, &err)
}

fn agent_session_event_public_payload(event: &SessionEvent) -> Value {
    match event {
        SessionEvent::ContextLoaded {
            session_id,
            product_ids,
            refs,
            product_count,
            knowledge_count,
            source_count,
            summary,
            sources_json,
        } => context_loaded_public_payload(ContextLoadedPublicPayloadRef {
            session_id: &session_id.0,
            product_ids,
            refs,
            product_count: *product_count,
            knowledge_count: *knowledge_count,
            source_count: *source_count,
            summary: summary.as_deref(),
            sources_json: sources_json.as_deref(),
        }),
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
        } => skill_result_public_payload(SkillResultPublicPayloadRef {
            session_id: &session_id.0,
            worker_id: &worker_id.0,
            task_id: &task_id.0,
            request_id,
            skill_id,
            role,
            status,
            summary: summary.as_deref(),
            error: error.as_deref(),
            output: output.as_ref(),
            execution_mode: execution_mode.as_deref(),
            risk_level: risk_level.as_deref(),
            approval_required: *approval_required,
        }),
        SessionEvent::TaskNodeReported {
            session_id,
            graph_id,
            node_id,
            worker_id,
            report,
        } => {
            let role_output = report.role_output.as_ref().map(|role_output| {
                json!({
                    "summary_available": !role_output.summary.trim().is_empty(),
                    "finding_count": role_output.findings.len(),
                    "recommendation_count": role_output.recommendations.len(),
                    "evidence_count": role_output.evidence.len(),
                    "risk_count": role_output.risks.len(),
                    "open_question_count": role_output.open_questions.len(),
                })
            });
            let diagnostic_kinds = report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind.clone())
                .collect::<Vec<_>>();
            task_node_reported_public_payload(TaskNodeReportedPublicPayloadRef {
                session_id: &session_id.0,
                graph_id: &graph_id.0,
                node_id: &node_id.0,
                worker_id: &worker_id.0,
                task_id: &report.task_id.0,
                role: &report.role,
                status: worker_report_status_code(&report.status),
                content_available: !report.content.trim().is_empty(),
                role_output,
                evidence_count: report.evidence.len(),
                risk_count: report.risks.len(),
                open_question_count: report.open_questions.len(),
                diagnostic_count: report.diagnostics.len(),
                diagnostic_kinds,
            })
        }
        _ => {
            let event_name = event.event_name();
            serde_json::to_value(event).unwrap_or_else(|_| json!({ "type": event_name }))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ContextLoadedPublicPayloadRef<'a> {
    session_id: &'a str,
    product_ids: &'a [String],
    refs: &'a [String],
    product_count: usize,
    knowledge_count: usize,
    source_count: usize,
    summary: Option<&'a str>,
    sources_json: Option<&'a str>,
}

fn context_loaded_public_payload(payload: ContextLoadedPublicPayloadRef<'_>) -> Value {
    json!({
        "type": "context_loaded",
        "session_id": payload.session_id,
        "product_ids": payload.product_ids,
        "refs": payload.refs,
        "product_count": payload.product_count,
        "knowledge_count": payload.knowledge_count,
        "source_count": payload.source_count,
        "summary_available": payload
            .summary
            .is_some_and(|value| !value.trim().is_empty()),
        "sources_available": payload
            .sources_json
            .is_some_and(|value| !value.trim().is_empty()),
    })
}

#[derive(Debug, Clone, Copy)]
struct SkillResultPublicPayloadRef<'a> {
    session_id: &'a str,
    worker_id: &'a str,
    task_id: &'a str,
    request_id: &'a str,
    skill_id: &'a str,
    role: &'a str,
    status: &'a str,
    summary: Option<&'a str>,
    error: Option<&'a str>,
    output: Option<&'a Value>,
    execution_mode: Option<&'a str>,
    risk_level: Option<&'a str>,
    approval_required: Option<bool>,
}

fn skill_result_public_payload(payload: SkillResultPublicPayloadRef<'_>) -> Value {
    json!({
        "type": "skill_result",
        "session_id": payload.session_id,
        "worker_id": payload.worker_id,
        "task_id": payload.task_id,
        "request_id": payload.request_id,
        "skill_id": payload.skill_id,
        "role": payload.role,
        "status": payload.status,
        "summary": payload.summary,
        "error": payload.error,
        "output_available": payload.output.is_some(),
        "output_summary": skill_output_public_summary(payload.output),
        "execution_mode": payload.execution_mode,
        "risk_level": payload.risk_level,
        "approval_required": payload.approval_required,
    })
}

#[derive(Debug)]
struct TaskNodeReportedPublicPayloadRef<'a> {
    session_id: &'a str,
    graph_id: &'a str,
    node_id: &'a str,
    worker_id: &'a str,
    task_id: &'a str,
    role: &'a str,
    status: &'a str,
    content_available: bool,
    role_output: Option<Value>,
    evidence_count: usize,
    risk_count: usize,
    open_question_count: usize,
    diagnostic_count: usize,
    diagnostic_kinds: Vec<String>,
}

fn task_node_reported_public_payload(payload: TaskNodeReportedPublicPayloadRef<'_>) -> Value {
    json!({
        "type": "task_node_reported",
        "session_id": payload.session_id,
        "graph_id": payload.graph_id,
        "node_id": payload.node_id,
        "worker_id": payload.worker_id,
        "report": {
            "graph_id": payload.graph_id,
            "node_id": payload.node_id,
            "task_id": payload.task_id,
            "worker_id": payload.worker_id,
            "role": payload.role,
            "status": payload.status,
            "content_available": payload.content_available,
            "role_output": payload.role_output,
            "evidence_count": payload.evidence_count,
            "risk_count": payload.risk_count,
            "open_question_count": payload.open_question_count,
            "diagnostic_count": payload.diagnostic_count,
            "diagnostic_kinds": payload.diagnostic_kinds,
        },
    })
}

fn skill_output_public_summary(output: Option<&Value>) -> Value {
    let Some(output) = output else {
        return json!({
            "kind": "none",
            "available": false,
        });
    };

    let object = output.as_object();
    let evidence_packet = object.and_then(|object| {
        object
            .get("evidence_packet")
            .or_else(|| object.get("evidencePacket"))
    });
    json!({
        "kind": output_kind(output),
        "available": true,
        "top_level_key_count": object.map_or(0, serde_json::Map::len),
        "status": output_string_field(output, &["status"]),
        "adapter_status": output_string_field(output, &["adapter_status", "adapterStatus"]),
        "data_readiness": output_string_field(output, &["data_readiness", "dataReadiness"]),
        "evidence_packet": evidence_packet_public_summary(evidence_packet),
    })
}

fn evidence_packet_public_summary(packet: Option<&Value>) -> Value {
    let Some(packet) = packet else {
        return json!({
            "available": false,
        });
    };
    json!({
        "available": true,
        "sources": value_array_len(packet, &["sources"]),
        "usable_claims": value_array_len(packet, &["usable_claims", "usableClaims"]),
        "gaps": value_array_len(packet, &["gaps"]),
        "confidence": packet.get("confidence").and_then(Value::as_f64),
        "search_strength": output_string_field(packet, &["search_strength", "searchStrength"]),
        "source_state": output_string_field(packet, &["source_state", "sourceState"]),
    })
}

fn output_kind(value: &Value) -> &'static str {
    if value.is_object() {
        "object"
    } else if value.is_array() {
        "array"
    } else if value.is_string() {
        "string"
    } else if value.is_number() {
        "number"
    } else if value.is_boolean() {
        "boolean"
    } else {
        "null"
    }
}

fn output_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn value_array_len(value: &Value, keys: &[&str]) -> usize {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_array))
        .map_or(0, Vec::len)
}

fn worker_report_status_code(status: &WorkerReportStatus) -> &'static str {
    match status {
        WorkerReportStatus::Completed => "completed",
        WorkerReportStatus::Failed => "failed",
        WorkerReportStatus::TimedOut => "timed_out",
    }
}

fn workspace_run_task_updated_public_payload(task: &WorkspaceRunTaskResponse) -> Value {
    json!({
        "id": task.id,
        "run_id": task.run_id,
        "workspace_id": task.workspace_id,
        "workspace_task_id": task.workspace_task_id,
        "seq": task.seq,
        "owner_role": task.owner_role,
        "status": task.status,
        "attempts": task.attempts,
        "acceptance_score": task.acceptance_score,
        "acceptance_passed": task.acceptance_passed,
        "acceptance_notes_available": task
            .acceptance_notes
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "execution_notes_available": task
            .execution_notes
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "metadata_available": object_has_fields(&task.metadata),
        "created_at": task.created_at,
        "updated_at": task.updated_at,
    })
}

fn workspace_run_public_payload(run: &WorkspaceRunResponse) -> Value {
    json!({
        "id": run.id,
        "workspace_id": run.workspace_id,
        "status": run.status,
        "source": run.source,
        "summary_available": run
            .summary
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "report_available": object_has_fields(&run.report),
        "metadata_available": object_has_fields(&run.metadata),
        "created_at": run.created_at,
        "updated_at": run.updated_at,
    })
}

fn workspace_artifact_created_public_payload(artifact: &WorkspaceArtifactResponse) -> Value {
    json!({
        "id": artifact.id,
        "workspace_id": artifact.workspace_id,
        "run_id": artifact.run_id,
        "workspace_task_id": artifact.workspace_task_id,
        "run_task_id": artifact.run_task_id,
        "artifact_type": artifact.artifact_type,
        "title": artifact.title,
        "content_available": artifact
            .content
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "preview_available": artifact
            .preview
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "mime_type": artifact.mime_type,
        "format": artifact.format,
        "storage_kind": artifact.storage_kind,
        "file_name": artifact.file_name,
        "file_size": artifact.file_size,
        "source": artifact.source,
        "status": artifact.status,
        "validation_status": artifact.validation_status,
        "validation_report_available": object_has_fields(&artifact.validation_report),
        "metadata_available": object_has_fields(&artifact.metadata),
        "created_at": artifact.created_at,
        "updated_at": artifact.updated_at,
    })
}

fn quality_checked_public_payload(evaluation: &AgentQualityEvaluationResponse) -> Value {
    let quality = &evaluation.quality_check;
    let trust = &evaluation.trust_update;
    json!({
        "quality_check": {
            "id": quality.id,
            "workspace_id": quality.workspace_id,
            "run_id": quality.run_id,
            "artifact_id": quality.artifact_id,
            "role": quality.role,
            "score": quality.score,
            "passed": quality.passed,
            "source": quality.source,
            "dimension_count": value_array_or_object_len(&quality.dimensions),
            "issue_count": value_array_or_object_len(&quality.issues),
            "suggestion_count": value_array_or_object_len(&quality.suggestions),
            "dimensions_available": object_has_fields(&quality.dimensions),
            "issues_available": value_has_items(&quality.issues),
            "suggestions_available": value_has_items(&quality.suggestions),
            "metadata_available": object_has_fields(&quality.metadata),
            "created_at": quality.created_at,
        },
        "trust_update": {
            "role": trust.role,
            "skill_id": trust.skill_id,
            "previous_score": trust.previous_score,
            "new_score": trust.new_score,
            "previous_level": trust.previous_level,
            "new_level": trust.new_level,
            "delta": trust.delta,
            "reason_available": !trust.reason.trim().is_empty(),
        },
        "learning_record_count": evaluation.learning_records.len(),
        "learning_records_available": !evaluation.learning_records.is_empty(),
    })
}

fn agent_learning_record_created_public_payload(record: &AgentLearningRecordResponse) -> Value {
    json!({
        "id": record.id,
        "workspace_id": record.workspace_id,
        "run_id": record.run_id,
        "quality_check_id": record.quality_check_id,
        "role": record.role,
        "category": record.category,
        "content_available": !record.content.trim().is_empty(),
        "confidence": record.confidence,
        "source_action": record.source_action,
        "feedback_rating": record.feedback_rating,
        "feedback_comment_available": record
            .feedback_comment
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty()),
        "status": record.status,
        "metadata_available": object_has_fields(&record.metadata),
        "created_at": record.created_at,
        "updated_at": record.updated_at,
    })
}

fn object_has_fields(value: &Value) -> bool {
    value.as_object().is_some_and(|object| !object.is_empty())
}

fn value_has_items(value: &Value) -> bool {
    match value {
        Value::Array(items) => !items.is_empty(),
        Value::Object(object) => !object.is_empty(),
        Value::String(value) => !value.trim().is_empty(),
        Value::Null => false,
        _ => true,
    }
}

fn value_array_or_object_len(value: &Value) -> usize {
    value
        .as_array()
        .map(Vec::len)
        .or_else(|| value.as_object().map(serde_json::Map::len))
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy)]
struct ApprovalRequiredPublicPayloadRef<'a> {
    skill_id: &'a str,
    request_id: &'a str,
    role: &'a str,
    risk_level: &'a str,
    reason: &'a str,
}

fn approval_required_public_payload(
    approval: &ToolApprovalResponse,
    alert: Option<&AlertResponse>,
    payload: ApprovalRequiredPublicPayloadRef<'_>,
) -> Value {
    json!({
        "skill_id": payload.skill_id,
        "request_id": payload.request_id,
        "role": payload.role,
        "risk_level": payload.risk_level,
        "reason": payload.reason,
        "approval": {
            "id": approval.id,
            "workspace_id": approval.workspace_id,
            "run_id": approval.run_id,
            "request_role": approval.request_role,
            "skill_name": approval.skill_name,
            "tool_call_id": approval.tool_call_id,
            "risk_level": approval.risk_level,
            "status": approval.status,
            "reason": approval.reason,
            "args_available": object_has_fields(&approval.args),
            "result_available": object_has_fields(&approval.result),
            "created_at": approval.created_at,
            "updated_at": approval.updated_at,
        },
        "alert": alert.map(|alert| {
            json!({
                "id": alert.id,
                "workspace_id": alert.workspace_id,
                "run_id": alert.run_id,
                "alert_type": alert.alert_type,
                "severity": alert.severity,
                "message": alert.message,
                "role": alert.role,
                "status": alert.status,
                "payload_available": object_has_fields(&alert.payload),
                "created_at": alert.created_at,
                "updated_at": alert.updated_at,
            })
        }),
    })
}

fn rag_context_loaded_public_payload(context: &AgentRetrievalContextResponse) -> Value {
    json!({
        "mode": context.mode,
        "query_terms": context.query_terms,
        "product_ids": context.product_ids,
        "knowledge_count": context.knowledge_count,
        "source_count": context.source_count,
        "summary_available": !context.summary.trim().is_empty(),
        "sources_available": !context.sources.is_empty(),
        "degraded": context.degraded,
        "degraded_reason": context.degraded_reason,
        "external_search_available": context.external_search_available,
        "external_search_configured": context.external_search_configured,
        "external_search_usable": context.external_search_available,
        "external_search_status": context.external_search_status,
    })
}

#[derive(Debug)]
struct CompletedAgentRunPublicPayloads {
    run: Option<Value>,
    artifact: Option<Value>,
    quality: Option<Value>,
    memory: Option<Value>,
    persistence_errors: Vec<Value>,
}

async fn finalize_completed_agent_run(
    link_state: AgentWorkspaceLinkState,
) -> CompletedAgentRunPublicPayloads {
    let workspace_api = WorkspaceApi::new(get_app_ctx());
    let mut persistence_errors = Vec::new();
    let response = workspace_api
        .complete_agent_run(CompleteAgentWorkspaceRunRequest {
            user_id: link_state.user_id.clone(),
            workspace_id: link_state.workspace_id.clone(),
            run_id: link_state.run_id.clone(),
            output: link_state.latest_output.clone().unwrap_or_default(),
        })
        .await;
    let completed = match response {
        Ok(value) => value,
        Err(err) => {
            persistence_errors.push(typed_persistence_error_payload(
                "complete_agent_run",
                err,
                &link_state,
            ));
            return CompletedAgentRunPublicPayloads {
                run: None,
                artifact: None,
                quality: None,
                memory: None,
                persistence_errors,
            };
        }
    };

    let run_payload = Some(workspace_run_public_payload(&completed.run));
    let artifact_payload = completed
        .artifact
        .as_ref()
        .map(workspace_artifact_created_public_payload);
    let artifact_id = completed
        .artifact
        .as_ref()
        .map(|artifact| artifact.id.clone());
    let quality_payload = match workspace_api
        .evaluate_agent_run(EvaluateAgentWorkspaceRunRequest {
            user_id: link_state.user_id.clone(),
            workspace_id: link_state.workspace_id.clone(),
            run_id: link_state.run_id.clone(),
            artifact_id,
            output: link_state.latest_output.clone(),
        })
        .await
    {
        Ok(value) => Some(quality_checked_public_payload(&value)),
        Err(err) => {
            persistence_errors.push(typed_persistence_error_payload(
                "evaluate_agent_run",
                err,
                &link_state,
            ));
            None
        }
    };

    let memory_payload = if let Some(conversation_id) = link_state.conversation_id.clone() {
        if let Some(memory) = build_completed_conversation_memory(&link_state) {
            match ConversationApi::new(get_app_ctx())
                .update_conversation_memory(
                    service::dto::conversation::UpdateConversationMemoryRequest {
                        user_id: link_state.user_id.clone(),
                        conversation_id: conversation_id.clone(),
                        memory: memory.clone(),
                    },
                )
                .await
            {
                Ok(_) => Some(conversation_memory_updated_public_payload(
                    &conversation_id,
                    &memory,
                )),
                Err(err) => {
                    persistence_errors.push(typed_persistence_error_payload(
                        "update_conversation_memory",
                        err,
                        &link_state,
                    ));
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    CompletedAgentRunPublicPayloads {
        run: run_payload,
        artifact: artifact_payload,
        quality: quality_payload,
        memory: memory_payload,
        persistence_errors,
    }
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

fn conversation_memory_updated_public_payload(conversation_id: &str, memory: &Value) -> Value {
    let last_user_input_available =
        memory_string_available(memory, "lastUserInput", "last_user_input");
    let last_assistant_output_available =
        memory_string_available(memory, "lastAssistantOutput", "last_assistant_output");
    json!({
        "conversation_id": conversation_id,
        "persisted": true,
        "memory_summary": {
            "kind": memory
                .get("kind")
                .or_else(|| memory.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("conversation_memory_summary"),
            "rolling_summary_available": memory_string_available(
                memory,
                "rollingSummary",
                "rolling_summary",
            ),
            "fact_count": memory_array_len(memory, "facts", "facts"),
            "decision_count": memory_array_len(memory, "decisions", "decisions"),
            "constraint_count": memory_array_len(memory, "constraints", "constraints"),
            "open_question_count": memory_array_len(memory, "openQuestions", "open_questions"),
            "last_turn_available": last_user_input_available || last_assistant_output_available,
            "last_user_input_available": last_user_input_available,
            "last_assistant_output_available": last_assistant_output_available,
            "source_map_available": memory
                .get("sourceMap")
                .or_else(|| memory.get("source_map"))
                .is_some_and(Value::is_object),
        }
    })
}

fn memory_string_available(memory: &Value, camel_key: &str, snake_key: &str) -> bool {
    memory
        .get(camel_key)
        .or_else(|| memory.get(snake_key))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn memory_array_len(memory: &Value, camel_key: &str, snake_key: &str) -> usize {
    memory
        .get(camel_key)
        .or_else(|| memory.get(snake_key))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
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
    metadata.insert(
        "rag.external_search_configured".to_string(),
        context.external_search_configured.to_string(),
    );
    metadata.insert(
        "rag.external_search_usable".to_string(),
        context.external_search_available.to_string(),
    );
    metadata.insert(
        "rag.external_search_status".to_string(),
        context.external_search_status.clone(),
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
    include_grounding: bool,
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
    metadata.insert(
        "conversation.history_grounding_enabled".to_string(),
        include_grounding.to_string(),
    );
    if !include_grounding {
        return summary;
    }
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
            "rollingSummaryAvailable": memory_string_available(
                &summary.memory_json,
                "rollingSummary",
                "rolling_summary",
            ),
            "factCount": memory_array_len(&summary.memory_json, "facts", "facts"),
            "decisionCount": memory_array_len(&summary.memory_json, "decisions", "decisions"),
            "constraintCount": memory_array_len(&summary.memory_json, "constraints", "constraints"),
            "openQuestionCount": memory_array_len(
                &summary.memory_json,
                "openQuestions",
                "open_questions",
            ),
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
    let older_summary = build_older_conversation_summary(&candidates[..recent_start]);
    let relevant_older =
        select_relevant_older_conversation_messages(&candidates[..recent_start], &current_input);
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
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{3400}'..='\u{4dbf}').contains(&ch)
        || ('\u{f900}'..='\u{faff}').contains(&ch)
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

fn workspace_context_loaded_public_payload(context: &WorkspaceContextSummaryResponse) -> Value {
    json!({
        "workspace_id": context.workspace_id,
        "conversation_id": context.conversation_id,
        "product_id": context.product_id,
        "run_id": context.run_id,
        "readiness": context.readiness,
        "counts": context.counts,
        "missing": context.missing,
        "warnings": context.warnings,
        "source_count": workspace_context_sources(context).len(),
    })
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

fn workspace_context_pack_loaded_public_payload(context: &WorkspaceContextPackResponse) -> Value {
    let readable_item_count = context
        .items
        .iter()
        .filter(|item| {
            item.text_chars > 0 || item.text.as_deref().is_some_and(|text| !text.is_empty())
        })
        .count();
    let text_chars = context
        .items
        .iter()
        .map(|item| item.text_chars)
        .sum::<usize>();
    let omitted_details = context
        .omitted_details
        .iter()
        .map(|item| {
            json!({
                "ref_key": item.ref_key,
                "reason": item.reason,
                "source_type": item.source_type,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "workspace_id": context.workspace_id,
        "conversation_id": context.conversation_id,
        "product_id": context.product_id,
        "run_id": context.run_id,
        "readiness": context.readiness,
        "counts": context.counts,
        "item_count": context.items.len(),
        "readable_item_count": readable_item_count,
        "text_chars": text_chars,
        "omitted": context.omitted,
        "omitted_details": omitted_details,
        "warnings": context.warnings,
    })
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
    use crate::dto::app::CreateProductDraftRequest;

    fn test_team_usage_preflight() -> AgentQuestionPreflight {
        AgentQuestionPreflight {
            user_id: Uuid::nil(),
            school_name: "测试学校".to_string(),
            team_name: "测试队伍".to_string(),
            question_limit: 100,
            question_used: 0,
            session_id: "session-1".to_string(),
            agent_id: Some("web-agent".to_string()),
            conversation_id: None,
        }
    }

    fn product_request(mode: Option<&str>) -> FormDraftStreamRequest {
        FormDraftStreamRequest {
            form_type: "product".to_string(),
            product: Some(CreateProductDraftRequest {
                name: Some("温显杯".to_string()),
                platform: Some("douyin".to_string()),
                category: None,
                brand: None,
                description: Some("防漏温显".to_string()),
                price: None,
                currency: None,
                prompt: None,
                generation_mode: mode.map(str::to_string),
            }),
            campaign: None,
        }
    }

    #[test]
    fn form_draft_stream_requires_agent_mode() {
        let err = form_draft_stream_target(product_request(None))
            .expect_err("template mode must not enter SSE");

        assert!(err.contains("generationMode=agent_llm"));
    }

    #[test]
    fn form_draft_stream_builds_agent_input_without_extra_payload() {
        let target = form_draft_stream_target(product_request(Some("agent_llm")))
            .expect("agent mode should parse");

        assert_eq!(target.kind, FormDraftStreamKind::Product);
        assert_eq!(target.input.kind.as_str(), "product");
        assert!(target.input.user_input.contains("温显杯"));
    }

    #[test]
    fn form_draft_process_payload_hides_model_details() {
        let payload = form_draft_agent_started_payload("product");
        let text = payload.to_string();

        assert_eq!(
            payload["systemAgentId"].as_str(),
            Some("form_draft_generator")
        );
        assert!(!text.contains("base_url"));
        assert!(!text.contains("api_key"));
        assert!(!text.contains("prompt"));
        assert!(!text.contains("model"));
    }

    #[test]
    fn form_draft_input_preflight_payload_uses_safe_summary_only() {
        let mut req = product_request(Some("agent_llm"));
        req.product.as_mut().expect("product payload").prompt = Some(
            "provider raw error model=hidden-model base_url=http://127.0.0.1:11434 prompt=zh/form_drafts/product_form_draft.md"
                .to_string(),
        );
        let target = form_draft_stream_target(req).expect("agent mode should parse");
        let payload = form_draft_input_preflighted_payload("product", &target.input);
        let text = payload.to_string();

        assert_eq!(payload["formType"].as_str(), Some("product"));
        assert_eq!(payload["generationMode"].as_str(), Some("agent_llm"));
        assert_eq!(payload["directWrite"].as_bool(), Some(false));
        assert_eq!(payload["requiresUserReview"].as_bool(), Some(true));
        assert_eq!(
            payload["inputSummary"]["existingFieldCount"].as_u64(),
            Some(3)
        );
        assert_eq!(
            payload["inputSummary"]["userInputAvailable"].as_bool(),
            Some(true)
        );
        assert_eq!(
            payload["inputSummary"]["contextSummaryAvailable"].as_bool(),
            Some(true)
        );
        assert!(!text.contains("温显杯"));
        assert!(!text.contains("防漏温显"));
        assert!(!text.contains("douyin"));
        assert!(!text.contains("hidden-model"));
        assert!(!text.contains("127.0.0.1"));
        assert!(!text.contains("product_form_draft"));
        assert!(!text.contains("prompt"));
        assert!(!text.contains("base_url"));
        assert!(!text.contains("api_key"));
        assert!(!text.contains("provider"));
        assert!(!text.contains("model"));
    }

    #[test]
    fn form_draft_failed_payload_keeps_safe_failure_summary() {
        let payload = form_draft_failed_payload("campaign", "agent_run_failed");
        let text = payload.to_string();

        assert_eq!(payload["formType"].as_str(), Some("campaign"));
        assert_eq!(payload["failureKind"].as_str(), Some("agent_run_failed"));
        assert_eq!(payload["messageAvailable"].as_bool(), Some(true));
        assert_eq!(payload["directWrite"].as_bool(), Some(false));
        assert_eq!(payload["requiresUserReview"].as_bool(), Some(true));
        assert_eq!(
            payload["draftQuality"]["invalidModelJsonPolicy"].as_str(),
            Some("reject_before_form_fill")
        );
        assert_eq!(payload["draftQuality"]["validated"].as_bool(), Some(false));
        assert!(!text.contains("base_url"));
        assert!(!text.contains("api_key"));
        assert!(!text.contains("prompt"));
        assert!(!text.contains("provider"));
        assert!(!text.contains("model"));
    }

    #[test]
    fn form_draft_review_ready_payload_keeps_user_review_boundary() {
        let payload = form_draft_review_ready_payload("product");
        let text = payload.to_string();

        assert_eq!(payload["formType"].as_str(), Some("product"));
        assert_eq!(payload["generationMode"].as_str(), Some("agent_llm"));
        assert_eq!(payload["directWrite"].as_bool(), Some(false));
        assert_eq!(payload["requiresUserReview"].as_bool(), Some(true));
        assert_eq!(
            payload["saveBoundary"]["directWrite"].as_bool(),
            Some(false)
        );
        assert_eq!(
            payload["saveBoundary"]["requiresUserReview"].as_bool(),
            Some(true)
        );
        assert!(!text.contains("base_url"));
        assert!(!text.contains("api_key"));
        assert!(!text.contains("prompt"));
        assert!(!text.contains("provider"));
        assert!(!text.contains("model"));
        assert!(!text.contains("draft"));
    }

    #[test]
    fn form_draft_mapped_success_events_emit_review_ready_before_completed() {
        let events = form_draft_mapped_success_events(
            "product",
            json!({
                "draft": {
                    "name": "raw-ready-draft"
                },
                "generationMode": "agent_llm"
            }),
        );

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "form_draft_review_ready");
        assert_eq!(events[1].0, "form_draft_completed");
        assert!(events[1].1["response"]["draft"]["name"].as_str().is_some());

        let review_ready_text = events[0].1.to_string();
        assert!(!review_ready_text.contains("raw-ready-draft"));
        assert!(!review_ready_text.contains("draft"));
        assert_eq!(events[0].1["requiresUserReview"].as_bool(), Some(true));
        assert_eq!(events[0].1["directWrite"].as_bool(), Some(false));
    }

    #[test]
    fn implicit_conversation_files_are_disabled_for_independent_turns() {
        let context = AgentStreamContextSelection {
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        assert!(!should_include_implicit_conversation_files(
            "新话题，不引用任何前面商品、活动、图片或文件：给学生一个通用判断框架。",
            &context,
        ));
    }

    #[test]
    fn implicit_conversation_files_are_disabled_when_turn_has_explicit_context() {
        let product_context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };
        let file_context = AgentStreamContextSelection {
            context_refs: vec!["conversation_file:file-1".to_string()],
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        assert!(!should_include_implicit_conversation_files(
            "分析这个商品",
            &product_context,
        ));
        assert!(!should_include_implicit_conversation_files(
            "分析这份文件",
            &file_context,
        ));
    }

    #[test]
    fn implicit_conversation_files_are_enabled_for_file_followups_only() {
        let context = AgentStreamContextSelection {
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        assert!(should_include_implicit_conversation_files(
            "继续看刚才上传的表格，退款率最高的问题是什么？",
            &context,
        ));
        assert!(!should_include_implicit_conversation_files(
            "学生开店第一周应该看什么指标？",
            &context,
        ));
    }

    #[test]
    fn conversation_history_grounding_is_disabled_for_independent_turns() {
        let context = AgentStreamContextSelection {
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        assert!(!should_include_conversation_history_grounding(
            "新话题，不引用任何前面商品、活动、图片或文件：给学生一个通用判断框架。",
            &context,
        ));
        assert!(!should_include_conversation_history_grounding(
            "学生开店第一周应该看什么指标？",
            &context,
        ));
    }

    #[test]
    fn conversation_history_grounding_is_enabled_for_followups_and_explicit_context() {
        let followup_context = AgentStreamContextSelection {
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };
        let product_context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        assert!(should_include_conversation_history_grounding(
            "继续同一个商品和活动，给我三条短视频开头文案。",
            &followup_context,
        ));
        assert!(should_include_conversation_history_grounding(
            "分析这个商品",
            &product_context,
        ));
    }

    #[test]
    fn generic_category_question_ignores_stale_explicit_context() {
        let context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            campaign_ids: vec!["campaign-1".to_string()],
            context_refs: vec![
                "product:product-1".to_string(),
                "conversation_file:file-1".to_string(),
            ],
            conversation_id: Some("conversation-1".to_string()),
        };

        let (effective, reason) = effective_context_selection_for_input(
            "大篇幅研究和回答各类商品，先给电商学生一个通用判断框架。",
            context,
        );

        assert_eq!(
            reason,
            Some("generic_category_question_without_selected_context_request")
        );
        assert!(effective.product_ids.is_empty());
        assert!(effective.campaign_ids.is_empty());
        assert!(effective.context_refs.is_empty());
        assert_eq!(effective.conversation_id.as_deref(), Some("conversation-1"));
    }

    #[test]
    fn selected_context_question_keeps_explicit_context() {
        let context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            context_refs: vec![
                "product:product-1".to_string(),
                "conversation_file:file-1".to_string(),
            ],
            conversation_id: Some("conversation-1".to_string()),
            ..AgentStreamContextSelection::default()
        };

        let (effective, reason) =
            effective_context_selection_for_input("分析这个商品，并结合这份资料。", context);

        assert!(reason.is_none());
        assert_eq!(effective.product_ids, vec!["product-1"]);
        assert_eq!(
            effective.context_refs,
            vec!["product:product-1", "conversation_file:file-1"]
        );
    }

    #[test]
    fn context_selection_metadata_includes_campaign_ids_and_refs() {
        let context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            campaign_ids: vec!["campaign-1".to_string()],
            context_refs: vec![
                "product:product-1".to_string(),
                "campaign:campaign-1".to_string(),
                "conversation_file:file-1".to_string(),
            ],
            ..AgentStreamContextSelection::default()
        };
        let mut metadata = std::collections::BTreeMap::new();

        merge_agent_stream_context_selection_metadata(&mut metadata, &context);

        assert_eq!(metadata["context.product_ids"], "product-1");
        assert_eq!(metadata["context.campaign_ids"], "campaign-1");
        assert_eq!(
            metadata["context.combined_refs"],
            "product:product-1,campaign:campaign-1,conversation_file:file-1"
        );
    }

    #[test]
    fn context_selection_metadata_stays_empty_after_generic_context_clear() {
        let context = AgentStreamContextSelection {
            product_ids: vec!["product-1".to_string()],
            campaign_ids: vec!["campaign-1".to_string()],
            context_refs: vec![
                "product:product-1".to_string(),
                "campaign:campaign-1".to_string(),
            ],
            ..AgentStreamContextSelection::default()
        };

        let (effective, reason) =
            effective_context_selection_for_input("各类商品都能怎么做推广？", context);
        let mut metadata = std::collections::BTreeMap::new();
        merge_agent_stream_context_selection_metadata(&mut metadata, &effective);

        assert_eq!(
            reason,
            Some("generic_category_question_without_selected_context_request")
        );
        assert!(!metadata.contains_key("context.product_ids"));
        assert!(!metadata.contains_key("context.campaign_ids"));
        assert!(!metadata.contains_key("context.combined_refs"));
    }

    #[test]
    fn conversation_history_metadata_can_skip_grounding_text() {
        let messages = vec![
            conversation_message(1, "user", "请分析这个商品"),
            conversation_message(
                2,
                "assistant",
                "上一轮分析了 QA矩阵智能保温杯 和 618 活动。",
            ),
        ];
        let mut metadata = std::collections::BTreeMap::new();

        let summary = merge_conversation_history_metadata(
            &mut metadata,
            "conversation-1",
            &messages,
            "新话题，不引用前面资料：给学生一个通用判断框架。",
            None,
            false,
        );

        assert_eq!(summary.total_message_count, 2);
        assert_eq!(metadata["conversation.history_grounding_enabled"], "false");
        assert!(!metadata.contains_key("conversation.history_summary"));
        assert!(
            !metadata
                .get("context.combined_summary")
                .is_some_and(|value| value.contains("QA矩阵智能保温杯"))
        );
    }

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
            external_search_configured: true,
            external_search_status: "configured".to_string(),
        };

        merge_agent_retrieval_metadata(&mut metadata, &context).expect("metadata should merge");

        assert!(metadata["context.summary"].contains("Selected context."));
        assert!(metadata["context.combined_summary"].contains("RAG summary."));
        assert_eq!(metadata["rag.source_count"], "1");
        assert_eq!(metadata["rag.external_search_available"], "true");
        assert_eq!(metadata["rag.external_search_configured"], "true");
        assert_eq!(metadata["rag.external_search_usable"], "true");
        assert_eq!(metadata["rag.external_search_status"], "configured");
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
            external_search_configured: false,
            external_search_status: "disabled".to_string(),
        };

        merge_agent_retrieval_metadata(&mut metadata, &context)
            .expect("degraded metadata should merge");

        assert!(metadata["context.combined_summary"].contains("no matching product knowledge"));
        assert!(
            metadata["context.combined_summary"].contains("Do not cite external search results")
        );
        assert_eq!(metadata["rag.degraded"], "true");
        assert_eq!(metadata["rag.external_search_available"], "false");
        assert_eq!(metadata["rag.external_search_configured"], "false");
        assert_eq!(metadata["rag.external_search_usable"], "false");
        assert_eq!(metadata["rag.external_search_status"], "disabled");
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
            external_search_configured: true,
            external_search_status: "configured".to_string(),
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
    fn degraded_retrieval_metadata_distinguishes_configured_but_unusable_search() {
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
                 configured but is currently unavailable."
                    .to_string(),
            ),
            external_search_available: false,
            external_search_configured: true,
            external_search_status: "cooldown".to_string(),
        };

        merge_agent_retrieval_metadata(&mut metadata, &context)
            .expect("degraded metadata should merge");

        assert_eq!(metadata["rag.external_search_available"], "false");
        assert_eq!(metadata["rag.external_search_configured"], "true");
        assert_eq!(metadata["rag.external_search_usable"], "false");
        assert_eq!(metadata["rag.external_search_status"], "cooldown");
        assert!(
            metadata["context.combined_summary"].contains("Do not cite external search results")
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
        for index in 1..=18 {
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
    fn conversation_context_loaded_event_hides_memory_text() {
        let hidden_summary = "隐藏的滚动摘要：用户强调预算和利润率。";
        let hidden_fact = "隐藏事实：预算不能超过 3000";
        let hidden_last_turn = "隐藏上一轮输出";
        let messages = vec![conversation_message(
            1,
            "user",
            "本轮补充：预算不能超过 3000",
        )];
        let previous_memory = json!({
            "kind": "conversation_memory_summary",
            "rollingSummary": hidden_summary,
            "facts": [
                {
                    "source": "conversation_message:old",
                    "text": hidden_fact,
                    "confidence": "explicit"
                }
            ],
            "decisions": [],
            "constraints": [],
            "openQuestions": [],
            "lastAssistantOutput": hidden_last_turn,
            "sourceMap": {
                "previousMemoryAvailable": true
            }
        });

        let summary = build_conversation_history_summary(&messages, "继续", Some(&previous_memory));
        let event = build_conversation_context_loaded_event("conversation-1", &summary, true);
        let serialized = event.to_string();

        assert_eq!(event["memory"]["rollingSummaryAvailable"], true);
        assert_eq!(event["memory"]["factCount"], 2);
        assert!(!serialized.contains(hidden_summary));
        assert!(!serialized.contains(hidden_fact));
        assert!(!serialized.contains(hidden_last_turn));
        assert!(!serialized.contains("\"rollingSummary\""));
        assert!(!serialized.contains("\"facts\""));
        assert!(!serialized.contains("\"sourceMap\""));
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
            team_usage_preflight: test_team_usage_preflight(),
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
    fn conversation_memory_updated_public_payload_hides_raw_memory() {
        let memory = json!({
            "kind": "conversation_memory_summary",
            "rollingSummary": "隐藏的滚动摘要",
            "facts": [
                {
                    "source": "conversation_message:1",
                    "text": "隐藏事实：预算不能超过 3000",
                    "confidence": "explicit"
                }
            ],
            "decisions": [
                {
                    "source": "agent_session:session-1",
                    "text": "隐藏决策：保留利润率判断",
                    "confidence": "generated"
                }
            ],
            "constraints": [
                {
                    "source": "conversation_message:2",
                    "text": "隐藏约束：不能降价",
                    "confidence": "explicit"
                }
            ],
            "openQuestions": [
                {
                    "source": "conversation_message:3",
                    "text": "隐藏问题：是否拆渠道",
                    "confidence": "open"
                }
            ],
            "lastUserInput": "隐藏最后输入",
            "lastAssistantOutput": "隐藏最后输出",
            "sourceMap": {
                "lastSessionId": "session-1",
                "persisted": true,
                "memoryBudget": {
                    "memoryTextMaxChars": 2400
                }
            }
        });

        let payload = camelize_json_keys(conversation_memory_updated_public_payload(
            "conversation-1",
            &memory,
        ));
        let serialized = payload.to_string();

        assert_eq!(payload["conversationId"], "conversation-1");
        assert_eq!(payload["persisted"], true);
        assert_eq!(
            payload["memorySummary"]["kind"],
            "conversation_memory_summary"
        );
        assert_eq!(payload["memorySummary"]["rollingSummaryAvailable"], true);
        assert_eq!(payload["memorySummary"]["factCount"], 1);
        assert_eq!(payload["memorySummary"]["decisionCount"], 1);
        assert_eq!(payload["memorySummary"]["constraintCount"], 1);
        assert_eq!(payload["memorySummary"]["openQuestionCount"], 1);
        assert_eq!(payload["memorySummary"]["lastTurnAvailable"], true);
        assert_eq!(payload["memorySummary"]["lastUserInputAvailable"], true);
        assert_eq!(
            payload["memorySummary"]["lastAssistantOutputAvailable"],
            true
        );
        assert_eq!(payload["memorySummary"]["sourceMapAvailable"], true);
        assert!(!serialized.contains("隐藏的滚动摘要"));
        assert!(!serialized.contains("隐藏事实"));
        assert!(!serialized.contains("隐藏决策"));
        assert!(!serialized.contains("隐藏约束"));
        assert!(!serialized.contains("隐藏问题"));
        assert!(!serialized.contains("隐藏最后输入"));
        assert!(!serialized.contains("隐藏最后输出"));
        assert!(!serialized.contains("\"rollingSummary\""));
        assert!(!serialized.contains("\"facts\""));
        assert!(!serialized.contains("\"decisions\""));
        assert!(!serialized.contains("\"constraints\""));
        assert!(!serialized.contains("\"openQuestions\""));
        assert!(!serialized.contains("\"lastUserInput\""));
        assert!(!serialized.contains("\"lastAssistantOutput\""));
        assert!(!serialized.contains("\"sourceMap\""));
        assert!(!serialized.contains("\"memoryBudget\""));
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
            team_usage_preflight: test_team_usage_preflight(),
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
            true,
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
    fn workspace_context_loaded_public_payload_hides_raw_sources() {
        let context = WorkspaceContextSummaryResponse {
            workspace_id: "workspace-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            product_id: Some("product-1".to_string()),
            run_id: None,
            readiness: "ready".to_string(),
            summary: "Hidden workspace summary body.".to_string(),
            counts: service::dto::workspace::WorkspaceContextSummaryCounts {
                campaigns: 1,
                conversation_files: 1,
                product_assets: 0,
                product_materials: 0,
                workspace_artifacts: 0,
            },
            attachment_sources: vec![WorkspaceContextSourceResponse {
                source_type: "conversation_file".to_string(),
                id: "file-1".to_string(),
                title: Some("hidden source title".to_string()),
                mime_type: Some("application/pdf".to_string()),
                file_size: Some(1024),
                status: "active".to_string(),
                ref_key: "conversation_file:file-1".to_string(),
                preview: Some("hidden source preview".to_string()),
                metadata: json!({"objectKey": "hidden/object/key.pdf"}),
            }],
            campaign_sources: vec![WorkspaceContextSourceResponse {
                source_type: "campaign".to_string(),
                id: "campaign-1".to_string(),
                title: Some("hidden campaign title".to_string()),
                mime_type: None,
                file_size: None,
                status: "active".to_string(),
                ref_key: "campaign:campaign-1".to_string(),
                preview: Some("hidden campaign preview".to_string()),
                metadata: json!({"secret": "hidden campaign metadata"}),
            }],
            product_asset_sources: Vec::new(),
            product_material_sources: Vec::new(),
            artifact_sources: Vec::new(),
            missing: vec!["product_material:missing".to_string()],
            warnings: vec!["1 source missing".to_string()],
            agent_context: json!({
                "kind": "workspace_context_summary",
                "refs": ["conversation_file:file-1"],
                "secret": "hidden agent context",
            }),
        };

        let value = typed_camel_event_value_for_test(
            "workspace_context_loaded",
            workspace_context_loaded_public_payload(&context),
        );
        let serialized = value.to_string();

        assert_eq!(value.get("sourceCount").and_then(Value::as_u64), Some(2));
        assert!(serialized.contains("product_material:missing"));
        assert!(!serialized.contains("Hidden workspace summary body"));
        assert!(!serialized.contains("hidden source title"));
        assert!(!serialized.contains("hidden source preview"));
        assert!(!serialized.contains("hidden/object/key"));
        assert!(!serialized.contains("hidden agent context"));
        assert!(value.get("agentContext").is_none());
        assert!(value.get("attachmentSources").is_none());
        assert!(value.get("campaignSources").is_none());
    }

    #[test]
    fn context_loaded_public_payload_hides_summary_and_sources_json() {
        let hidden_summary = "hidden selected product summary with margin and supplier details";
        let hidden_source_title = "hidden source title";
        let hidden_sources_json = json!([
            {
                "source_type": "product_knowledge",
                "id": "knowledge-1",
                "product_id": "product-1",
                "title": hidden_source_title
            }
        ])
        .to_string();
        let event = SessionEvent::ContextLoaded {
            session_id: agent::api::SessionId("session-1".to_string()),
            product_ids: vec!["product-1".to_string()],
            refs: vec!["product:product-1".to_string()],
            product_count: 1,
            knowledge_count: 2,
            source_count: 3,
            summary: Some(hidden_summary.to_string()),
            sources_json: Some(hidden_sources_json),
        };

        let value = camelize_json_keys(agent_session_event_public_payload(&event));
        let serialized = value.to_string();

        assert_eq!(value["type"], "context_loaded");
        assert_eq!(value["sessionId"], "session-1");
        assert_eq!(value["productIds"].as_array().map(Vec::len), Some(1));
        assert_eq!(value["refs"].as_array().map(Vec::len), Some(1));
        assert_eq!(value["productCount"], 1);
        assert_eq!(value["knowledgeCount"], 2);
        assert_eq!(value["sourceCount"], 3);
        assert_eq!(value["summaryAvailable"], true);
        assert_eq!(value["sourcesAvailable"], true);
        assert!(!serialized.contains(hidden_summary));
        assert!(!serialized.contains(hidden_source_title));
        assert!(!serialized.contains("sourcesJson"));
        assert!(value.get("summary").is_none());
        assert!(value.get("sourcesJson").is_none());
    }

    #[test]
    fn agent_output_public_payload_is_the_single_final_output_event_body() {
        let event = SessionEvent::Output {
            session_id: agent::api::SessionId("session-1".to_string()),
            content: "final answer".to_string(),
        };

        let value = typed_json_event_value_for_test(
            event.event_name(),
            agent_session_event_public_payload(&event),
        );

        assert_eq!(event.event_name(), "output");
        assert_eq!(value["type"], "output");
        assert_eq!(value["content"], "final answer");
    }

    #[test]
    fn agent_stream_startup_timeout_message_is_user_facing() {
        let message = agent_stream_startup_timeout_message(Duration::from_secs(60));
        let lower = message.to_ascii_lowercase();

        assert!(message.contains("当前 AI 会话在 60s 内没有成功进入执行阶段"));
        assert!(message.contains("本轮没有开始可靠的智能体协作"));
        assert!(message.contains("可以稍后重试同一个问题"));
        assert!(!lower.contains("agent stream startup timed out"));
        assert!(!lower.contains("workspace_run"));
        assert!(!lower.contains("session_id"));
    }

    #[test]
    fn rag_context_loaded_public_payload_hides_summary_and_source_titles() {
        let hidden_summary = "hidden rag summary with product knowledge details";
        let hidden_title = "hidden knowledge title";
        let context = AgentRetrievalContextResponse {
            mode: "local_product_knowledge".to_string(),
            query_terms: vec!["咖啡".to_string(), "转化".to_string()],
            product_ids: vec!["product-1".to_string()],
            knowledge_count: 2,
            source_count: 2,
            summary: hidden_summary.to_string(),
            sources: vec![AgentContextSource {
                source_type: "product_knowledge".to_string(),
                id: "knowledge-1".to_string(),
                product_id: Some("product-1".to_string()),
                title: hidden_title.to_string(),
            }],
            degraded: false,
            degraded_reason: None,
            external_search_available: true,
            external_search_configured: true,
            external_search_status: "configured".to_string(),
        };

        let value = typed_camel_event_value_for_test(
            "rag_context_loaded",
            rag_context_loaded_public_payload(&context),
        );
        let serialized = value.to_string();

        assert_eq!(value["type"], "rag_context_loaded");
        assert_eq!(value["mode"], "local_product_knowledge");
        assert_eq!(value["queryTerms"].as_array().map(Vec::len), Some(2));
        assert_eq!(value["productIds"].as_array().map(Vec::len), Some(1));
        assert_eq!(value["knowledgeCount"], 2);
        assert_eq!(value["sourceCount"], 2);
        assert_eq!(value["summaryAvailable"], true);
        assert_eq!(value["sourcesAvailable"], true);
        assert_eq!(value["externalSearchAvailable"], true);
        assert_eq!(value["externalSearchConfigured"], true);
        assert_eq!(value["externalSearchUsable"], true);
        assert_eq!(value["externalSearchStatus"], "configured");
        assert!(!serialized.contains(hidden_summary));
        assert!(!serialized.contains(hidden_title));
        assert!(value.get("summary").is_none());
        assert!(value.get("sources").is_none());
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
    fn workspace_context_pack_loaded_public_payload_hides_raw_item_bodies() {
        let context = WorkspaceContextPackResponse {
            workspace_id: "workspace-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            product_id: Some("product-1".to_string()),
            run_id: None,
            readiness: "partial".to_string(),
            summary: "Hidden workspace pack summary.".to_string(),
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
                    title: Some("hidden item title".to_string()),
                    ref_key: "product_material:material-1".to_string(),
                    content_kind: "content".to_string(),
                    text: Some("hidden pack readable text".to_string()),
                    text_chars: 25,
                    truncated: false,
                    metadata: json!({"secret": "hidden item metadata"}),
                },
                service::dto::workspace::WorkspaceContextPackItemResponse {
                    source_type: "conversation_file".to_string(),
                    id: "file-1".to_string(),
                    title: Some("hidden metadata only title".to_string()),
                    ref_key: "conversation_file:file-1".to_string(),
                    content_kind: "metadata_only".to_string(),
                    text: None,
                    text_chars: 0,
                    truncated: false,
                    metadata: json!({"objectKey": "hidden/file.pdf"}),
                },
            ],
            omitted: vec!["conversation_file:file-2".to_string()],
            omitted_details: vec![
                service::dto::workspace::WorkspaceContextPackOmittedResponse {
                    ref_key: "conversation_file:file-2".to_string(),
                    reason: "extraction_pending".to_string(),
                    source_type: "conversation_file".to_string(),
                    title: Some("hidden omitted title".to_string()),
                },
            ],
            warnings: vec!["1 conversation file is pending extraction".to_string()],
            agent_context: json!({
                "kind": "workspace_context_pack",
                "refs": ["product_material:material-1"],
                "secret": "hidden pack agent context",
            }),
        };

        let value = typed_camel_event_value_for_test(
            "workspace_context_pack_loaded",
            workspace_context_pack_loaded_public_payload(&context),
        );
        let serialized = value.to_string();

        assert_eq!(value.get("itemCount").and_then(Value::as_u64), Some(2));
        assert_eq!(
            value.get("readableItemCount").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(value.get("textChars").and_then(Value::as_u64), Some(25));
        assert!(serialized.contains("conversation_file:file-2"));
        assert!(serialized.contains("extraction_pending"));
        assert!(!serialized.contains("Hidden workspace pack summary"));
        assert!(!serialized.contains("hidden pack readable text"));
        assert!(!serialized.contains("hidden item title"));
        assert!(!serialized.contains("hidden item metadata"));
        assert!(!serialized.contains("hidden omitted title"));
        assert!(!serialized.contains("hidden pack agent context"));
        assert!(value.get("items").is_none());
        assert!(value.get("agentContext").is_none());
    }

    #[test]
    fn skill_result_public_payload_hides_raw_output_but_keeps_safe_counts() {
        let value = typed_camel_event_value_for_test(
            "skill_result",
            skill_result_public_payload(SkillResultPublicPayloadRef {
                session_id: "session-1",
                worker_id: "worker.search",
                task_id: "task-1",
                request_id: "request-1",
                skill_id: "search_competitor",
                role: "search",
                status: "success",
                summary: Some("search completed"),
                error: None,
                output: Some(&json!({
                    "adapter_status": "live",
                    "planned_query": "hidden search query",
                    "sources": [{"title": "hidden source title"}],
                    "evidence_packet": {
                        "confidence": 0.72,
                        "search_strength": "standard",
                        "sources": [{"title": "hidden packet source"}],
                        "usable_claims": ["hidden claim"],
                        "gaps": ["hidden gap"]
                    },
                    "data": {
                        "items": [
                            {"published_at": "2026-05-10T00:00:00Z"}
                        ]
                    }
                })),
                execution_mode: Some("adapter"),
                risk_level: Some("low"),
                approval_required: Some(false),
            }),
        );
        let serialized = value.to_string();

        assert_eq!(value["skillId"], "search_competitor");
        assert_eq!(value["outputAvailable"], true);
        assert_eq!(value["outputSummary"]["adapterStatus"], "live");
        assert_eq!(value["outputSummary"]["evidencePacket"]["sources"], 1);
        assert_eq!(value["outputSummary"]["evidencePacket"]["usableClaims"], 1);
        assert_eq!(value["outputSummary"]["evidencePacket"]["gaps"], 1);
        assert_eq!(value["outputSummary"]["evidencePacket"]["confidence"], 0.72);
        assert!(value.get("output").is_none());
        assert!(!serialized.contains("hidden search query"));
        assert!(!serialized.contains("hidden source title"));
        assert!(!serialized.contains("hidden packet source"));
        assert!(!serialized.contains("hidden claim"));
        assert!(!serialized.contains("publishedAt"));
    }

    #[test]
    fn task_node_reported_public_payload_hides_raw_report_text_and_diagnostics() {
        let role_output = Some(json!({
            "summary_available": true,
            "finding_count": 2,
            "recommendation_count": 1,
            "evidence_count": 1,
            "risk_count": 1,
            "open_question_count": 1,
        }));
        let value = typed_camel_event_value_for_test(
            "task_node_reported",
            task_node_reported_public_payload(TaskNodeReportedPublicPayloadRef {
                session_id: "session-1",
                graph_id: "graph-1",
                node_id: "node-1",
                worker_id: "worker.data",
                task_id: "task-1",
                role: "data",
                status: "completed",
                content_available: true,
                role_output,
                evidence_count: 3,
                risk_count: 1,
                open_question_count: 2,
                diagnostic_count: 1,
                diagnostic_kinds: vec!["search_evidence".to_string()],
            }),
        );
        let serialized = value.to_string();

        assert_eq!(value["report"]["role"], "data");
        assert_eq!(value["report"]["contentAvailable"], true);
        assert_eq!(value["report"]["evidenceCount"], 3);
        assert_eq!(value["report"]["diagnosticKinds"][0], "search_evidence");
        assert!(value["report"].get("content").is_none());
        assert!(value["report"].get("evidence").is_none());
        assert!(value["report"].get("risks").is_none());
        assert!(value["report"].get("diagnostics").is_none());
        assert!(!serialized.contains("hidden report body"));
        assert!(!serialized.contains("hidden diagnostic reason"));
    }

    #[test]
    fn workspace_persistence_public_payloads_hide_raw_content_and_metadata() {
        let now = time::OffsetDateTime::UNIX_EPOCH;
        let task = WorkspaceRunTaskResponse {
            id: "run-task-1".to_string(),
            run_id: "run-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            workspace_task_id: Some("workspace-task-1".to_string()),
            user_id: "user-1".to_string(),
            seq: 1,
            owner_role: Some("data".to_string()),
            status: "succeeded".to_string(),
            attempts: 1,
            acceptance_score: 0.8,
            acceptance_passed: true,
            acceptance_notes: Some("hidden acceptance notes".to_string()),
            execution_notes: Some("hidden execution notes".to_string()),
            metadata: json!({
                "report": {
                    "content": "hidden persisted report body",
                    "roleOutput": {"summary": "hidden role summary"}
                }
            }),
            created_at: now,
            updated_at: now,
        };
        let artifact = WorkspaceArtifactResponse {
            id: "artifact-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            run_id: Some("run-1".to_string()),
            workspace_task_id: None,
            run_task_id: Some("run-task-1".to_string()),
            user_id: "user-1".to_string(),
            artifact_type: "search_evidence".to_string(),
            title: "Search evidence".to_string(),
            content: Some("hidden artifact content".to_string()),
            preview: Some("hidden artifact preview".to_string()),
            mime_type: Some("application/json".to_string()),
            format: "json".to_string(),
            storage_kind: "inline".to_string(),
            bucket: Some("hidden-bucket".to_string()),
            object_key: Some("hidden/object/key.json".to_string()),
            file_name: Some("evidence.json".to_string()),
            file_size: Some(128),
            sha256: Some("hidden-sha".to_string()),
            source: "agent_skill_result".to_string(),
            status: "active".to_string(),
            validation_status: "unchecked".to_string(),
            validation_report: json!({"secret": "hidden validation report"}),
            metadata: json!({"secret": "hidden artifact metadata"}),
            created_at: now,
            updated_at: now,
        };
        let record = AgentLearningRecordResponse {
            id: "learning-1".to_string(),
            user_id: "user-1".to_string(),
            workspace_id: Some("workspace-1".to_string()),
            run_id: Some("run-1".to_string()),
            quality_check_id: None,
            role: "ops".to_string(),
            category: "decision".to_string(),
            content: "hidden learning content".to_string(),
            confidence: 0.88,
            source_action: "coordination_save_memory".to_string(),
            feedback_rating: None,
            feedback_comment: Some("hidden feedback".to_string()),
            status: "active".to_string(),
            metadata: json!({"secret": "hidden learning metadata"}),
            created_at: now,
            updated_at: now,
        };

        let task_value = typed_camel_event_value_for_test(
            "workspace_run_task_updated",
            workspace_run_task_updated_public_payload(&task),
        );
        let artifact_value = typed_camel_event_value_for_test(
            "workspace_artifact_created",
            workspace_artifact_created_public_payload(&artifact),
        );
        let record_value = typed_camel_event_value_for_test(
            "agent_learning_record_created",
            agent_learning_record_created_public_payload(&record),
        );
        let serialized = json!([task_value, artifact_value, record_value]).to_string();

        assert_eq!(artifact_value["contentAvailable"], true);
        assert_eq!(artifact_value["metadataAvailable"], true);
        assert_eq!(record_value["contentAvailable"], true);
        assert_eq!(task_value["metadataAvailable"], true);
        assert!(!serialized.contains("hidden persisted report body"));
        assert!(!serialized.contains("hidden acceptance notes"));
        assert!(!serialized.contains("hidden execution notes"));
        assert!(!serialized.contains("hidden artifact content"));
        assert!(!serialized.contains("hidden artifact preview"));
        assert!(!serialized.contains("hidden/object/key"));
        assert!(!serialized.contains("hidden validation report"));
        assert!(!serialized.contains("hidden artifact metadata"));
        assert!(!serialized.contains("hidden learning content"));
        assert!(!serialized.contains("hidden learning metadata"));
        assert!(artifact_value.get("objectKey").is_none());
        assert!(artifact_value.get("bucket").is_none());
        assert!(artifact_value.get("content").is_none());
        assert!(artifact_value.get("metadata").is_none());
        assert!(record_value.get("content").is_none());
        assert!(record_value.get("metadata").is_none());
        assert!(task_value.get("metadata").is_none());
    }

    #[test]
    fn completion_and_quality_public_payloads_hide_raw_report_and_quality_details() {
        let now = time::OffsetDateTime::UNIX_EPOCH;
        let run = WorkspaceRunResponse {
            id: "run-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            user_id: "user-1".to_string(),
            goal: "hidden run goal".to_string(),
            requirements: json!({"input": "hidden user input"}),
            status: "succeeded".to_string(),
            source: "agent_session".to_string(),
            summary: Some("hidden final summary".to_string()),
            report: json!({
                "output": "hidden final answer",
                "tasks": [{"content": "hidden task report"}],
            }),
            metadata: json!({"artifactId": "artifact-1", "hidden": "run metadata"}),
            created_at: now,
            updated_at: now,
        };
        let quality = AgentQualityEvaluationResponse {
            quality_check: service::dto::workspace::AgentQualityCheckResponse {
                id: "quality-1".to_string(),
                workspace_id: "workspace-1".to_string(),
                run_id: "run-1".to_string(),
                artifact_id: Some("artifact-1".to_string()),
                user_id: "user-1".to_string(),
                role: "agent".to_string(),
                score: 0.82,
                passed: false,
                dimensions: json!({"clarity": 0.8, "grounding": 0.7}),
                issues: json!(["hidden quality issue"]),
                suggestions: json!(["hidden quality suggestion"]),
                source: "agent_session".to_string(),
                metadata: json!({"hidden": "quality metadata"}),
                created_at: now,
            },
            trust_update: service::dto::workspace::AgentTrustUpdateResponse {
                role: "agent".to_string(),
                skill_id: None,
                previous_score: 0.5,
                new_score: 0.66,
                previous_level: "moderate".to_string(),
                new_level: "moderate".to_string(),
                delta: 0.16,
                reason: "hidden trust reason".to_string(),
            },
            learning_records: vec![AgentLearningRecordResponse {
                id: "learning-1".to_string(),
                user_id: "user-1".to_string(),
                workspace_id: Some("workspace-1".to_string()),
                run_id: Some("run-1".to_string()),
                quality_check_id: Some("quality-1".to_string()),
                role: "agent".to_string(),
                category: "quality_issue".to_string(),
                content: "hidden learning content".to_string(),
                confidence: 0.72,
                source_action: "agent_quality_check".to_string(),
                feedback_rating: None,
                feedback_comment: Some("hidden feedback comment".to_string()),
                status: "active".to_string(),
                metadata: json!({"hidden": "learning metadata"}),
                created_at: now,
                updated_at: now,
            }],
        };

        let run_value = typed_camel_event_value_for_test(
            "workspace_run_completed",
            workspace_run_public_payload(&run),
        );
        let quality_value = typed_camel_event_value_for_test(
            "quality_checked",
            quality_checked_public_payload(&quality),
        );
        let run_serialized = run_value.to_string();
        let quality_serialized = quality_value.to_string();

        assert_eq!(run_value["type"], "workspace_run_completed");
        assert_eq!(run_value["summaryAvailable"], true);
        assert_eq!(run_value["reportAvailable"], true);
        assert_eq!(run_value["metadataAvailable"], true);
        assert!(run_value.get("summary").is_none());
        assert!(run_value.get("report").is_none());
        assert!(run_value.get("requirements").is_none());
        assert!(!run_serialized.contains("hidden final answer"));
        assert!(!run_serialized.contains("hidden final summary"));
        assert!(!run_serialized.contains("hidden user input"));
        assert!(!run_serialized.contains("hidden task report"));
        assert!(!run_serialized.contains("run metadata"));

        assert_eq!(quality_value["type"], "quality_checked");
        assert_eq!(quality_value["qualityCheck"]["score"], 0.82);
        assert_eq!(quality_value["qualityCheck"]["passed"], false);
        assert_eq!(quality_value["qualityCheck"]["dimensionCount"], 2);
        assert_eq!(quality_value["qualityCheck"]["issueCount"], 1);
        assert_eq!(quality_value["qualityCheck"]["suggestionCount"], 1);
        assert_eq!(quality_value["qualityCheck"]["dimensionsAvailable"], true);
        assert_eq!(quality_value["qualityCheck"]["issuesAvailable"], true);
        assert_eq!(quality_value["qualityCheck"]["suggestionsAvailable"], true);
        assert_eq!(quality_value["qualityCheck"]["metadataAvailable"], true);
        assert_eq!(quality_value["trustUpdate"]["reasonAvailable"], true);
        assert_eq!(quality_value["learningRecordCount"], 1);
        assert_eq!(quality_value["learningRecordsAvailable"], true);
        assert!(quality_value["qualityCheck"].get("dimensions").is_none());
        assert!(quality_value["qualityCheck"].get("issues").is_none());
        assert!(quality_value["qualityCheck"].get("suggestions").is_none());
        assert!(quality_value["qualityCheck"].get("metadata").is_none());
        assert!(quality_value["trustUpdate"].get("reason").is_none());
        assert!(quality_value.get("learningRecords").is_none());
        assert!(!quality_serialized.contains("hidden quality issue"));
        assert!(!quality_serialized.contains("hidden quality suggestion"));
        assert!(!quality_serialized.contains("quality metadata"));
        assert!(!quality_serialized.contains("hidden trust reason"));
        assert!(!quality_serialized.contains("hidden learning content"));
        assert!(!quality_serialized.contains("hidden feedback comment"));
        assert!(!quality_serialized.contains("learning metadata"));
    }

    #[test]
    fn approval_required_public_payload_hides_raw_args_output() {
        let now = time::OffsetDateTime::UNIX_EPOCH;
        let approval = ToolApprovalResponse {
            id: "approval-1".to_string(),
            user_id: "user-1".to_string(),
            workspace_id: Some("workspace-1".to_string()),
            run_id: Some("run-1".to_string()),
            conversation_id: None,
            request_role: Some("ops".to_string()),
            skill_name: "platform_update_price".to_string(),
            tool_call_id: Some("request-1".to_string()),
            risk_level: "L3".to_string(),
            args: json!({
                "output": {
                    "secret": "hidden approval raw output"
                },
                "normalizedInputs": {
                    "price": 159
                }
            }),
            status: "pending".to_string(),
            reason: Some("needs approval".to_string()),
            action_key: Some("hidden-action-key".to_string()),
            result: json!({"executionReadiness": {"secret": "hidden result"}}),
            approved_by: None,
            approved_at: None,
            executed_at: None,
            created_at: now,
            updated_at: now,
        };
        let alert = AlertResponse {
            id: "alert-1".to_string(),
            user_id: "user-1".to_string(),
            workspace_id: Some("workspace-1".to_string()),
            run_id: Some("run-1".to_string()),
            alert_type: "agent_approval_required".to_string(),
            severity: "warning".to_string(),
            message: "approval required".to_string(),
            role: Some("ops".to_string()),
            status: "active".to_string(),
            dedup_key: Some("hidden-dedup-key".to_string()),
            snoozed_until: None,
            payload: json!({"secret": "hidden alert payload"}),
            acted_at: None,
            dismissed_at: None,
            created_at: now,
            updated_at: now,
        };
        let value = typed_camel_event_value_for_test(
            "approval_required",
            approval_required_public_payload(
                &approval,
                Some(&alert),
                ApprovalRequiredPublicPayloadRef {
                    skill_id: "platform_update_price",
                    request_id: "request-1",
                    role: "ops",
                    risk_level: "L3",
                    reason: "needs approval",
                },
            ),
        );
        let serialized = value.to_string();

        assert_eq!(value["approval"]["argsAvailable"], true);
        assert_eq!(value["approval"]["resultAvailable"], true);
        assert_eq!(value["alert"]["payloadAvailable"], true);
        assert!(value["approval"].get("args").is_none());
        assert!(value["approval"].get("result").is_none());
        assert!(value["approval"].get("actionKey").is_none());
        assert!(value["alert"].get("payload").is_none());
        assert!(value["alert"].get("dedupKey").is_none());
        assert!(!serialized.contains("hidden approval raw output"));
        assert!(!serialized.contains("hidden result"));
        assert!(!serialized.contains("hidden-action-key"));
        assert!(!serialized.contains("hidden-dedup-key"));
        assert!(!serialized.contains("hidden alert payload"));
    }

    #[test]
    fn persistence_error_public_payload_keeps_only_safe_error_fields() {
        let err = db_core::error::BizError::new(
            error_code::app::WORKSPACE_REQUEST_INVALID,
            "quality check persistence failed".to_string(),
        );
        let value = typed_json_event_value_for_test(
            "workspace_run_persistence_error",
            persistence_error_payload("evaluate_agent_run", &err),
        );

        assert_eq!(value["type"], "workspace_run_persistence_error");
        assert_eq!(value["action"], "evaluate_agent_run");
        assert_eq!(
            value["code"],
            i64::from(error_code::app::WORKSPACE_REQUEST_INVALID)
        );
        assert_eq!(value["message"], "quality check persistence failed");
        assert!(value.get("runId").is_none());
        assert!(value.get("sessionId").is_none());
        assert!(value.get("workspaceId").is_none());
        assert!(value.get("metadata").is_none());
    }

    fn typed_camel_event_value_for_test(event: &'static str, value: Value) -> Value {
        ensure_event_type(event, camelize_json_keys(value))
    }

    fn typed_json_event_value_for_test(event: &'static str, value: Value) -> Value {
        ensure_event_type(event, value)
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
            strategy_id: None,
            response_mode: None,
            routing_mode: None,
            collaboration_mode: None,
            learning_level: None,
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
