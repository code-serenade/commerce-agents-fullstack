#![allow(unused_imports)]

use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use file_ingest::ViewOptions;
use service::api::{
    app::AppApi,
    approval_audit::ApprovalAuditApi,
    briefing_ops::BriefingOpsApi,
    campaign::CampaignApi,
    conversation::ConversationApi,
    document_text_extraction_manager::{
        DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY, DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION,
    },
    package_governance::PackageGovernanceApi,
    platform_connect::PlatformConnectApi,
    product::ProductApi,
    product_asset::{ProductAssetApi, product_asset_object_key},
    product_competitor::ProductCompetitorApi,
    product_knowledge::ProductKnowledgeApi,
    product_material::ProductMaterialApi,
    vision_preprocess_manager::{
        EnsureVisionPreprocessRequest, VisionPreprocessConfig, VisionPreprocessManager,
    },
    workspace::WorkspaceApi,
};
use storage::{BucketKind, DownloadObjectRequest, PresignDownloadRequest};
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use utils::image::vision_image_data_url;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::app::{
        ActAlertRequest, AddConversationMessageRequest, AgentContextPreviewRequest,
        AgentContextPreviewResponse, AlertListResponse, AlertResponse, AlertSummaryResponse,
        AppUserResponse, AuditLogListResponse, AuditLogResponse, BackgroundEventListResponse,
        BackgroundEventResponse, BatchUpdateResponse, CampaignListResponse, CampaignResponse,
        CancelLongTaskRequest, ChannelCatalogResponse, ChannelConnectionListResponse,
        ChannelConnectionResponse, CompetitorChangeResponse, ConsumeBackgroundEventsRequest,
        ConversationDetailResponse, ConversationFileAccessResponse,
        ConversationFileExtractionJobResponse, ConversationFileExtractionStatusResponse,
        ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
        CreateAlertRequest, CreateAuditLogRequest, CreateBackgroundEventRequest,
        CreateCampaignRequest, CreateCompetitorChangeRequest, CreateConversationRequest,
        CreateExecutionQueueRequest, CreateLongTaskRequest, CreateProductCompetitorRequest,
        CreateProductKnowledgeRequest, CreateProductMaterialRequest, CreateProductRequest,
        CreateToolApprovalRequest, CreateWorkspaceRequest, CreateWorkspaceRunRequest,
        CreateWorkspaceRunTaskRequest, CreateWorkspaceTaskRequest, CurrentUserPermissionsResponse,
        DailyBriefingListResponse, DailyBriefingQuery, DailyBriefingResponse,
        DailyBriefingSummaryResponse, DecideExecutionQueueRequest, DecideToolApprovalRequest,
        DismissAlertRequest, DismissAllAlertsRequest, ExecutionQueueListResponse,
        ExecutionQueueResponse, ListAlertsQuery, ListAuditLogsQuery, ListBackgroundEventsQuery,
        ListCampaignsQuery, ListChannelConnectionsQuery, ListCompetitorChangesQuery,
        ListDailyBriefingRecordsQuery, ListExecutionQueueQuery, ListLongTasksQuery,
        ListPackageCatalogQuery, ListPackageHistoryQuery, ListPlatformConnectionsQuery,
        ListProductAssetsQuery, ListProductCompetitorsQuery, ListProductKnowledgeQuery,
        ListProductMaterialsQuery, ListProductsQuery, ListStoreMetricsQuery,
        ListToolApprovalsQuery, ListWorkspaceRunTasksQuery, ListWorkspaceRunsQuery,
        ListWorkspaceTasksQuery, ListWorkspacesQuery, LongTaskListResponse, LongTaskResponse,
        PackageCatalogResponse, PackageHistoryListResponse, PackageLockDriftResponse,
        PackageLockListResponse, PackageResponse, PackageStatusChangeRequest,
        PackageValidationResponse, PlatformCatalogResponse, PlatformConnectionListResponse,
        PlatformConnectionResponse, PlatformSummaryResponse, PlatformSyncStatusListResponse,
        PresignConversationFileUploadRequest, PresignConversationFileUploadResponse,
        PresignProductAssetUploadRequest, PresignProductAssetUploadResponse,
        ProductAssetAccessResponse, ProductAssetResponse, ProductCompetitorResponse,
        ProductDashboardResponse, ProductDetailResponse, ProductKnowledgeResponse,
        ProductMaterialResponse, ProductResponse, RegisterAppUserRequest,
        RegisterConversationFileRequest, RegisterProductAssetRequest, SaveChannelConnectionRequest,
        SavePackageLocksRequest, SavePlatformConnectionRequest, SaveStoreMetricRequest,
        SnoozeAlertRequest, StoreMetricListResponse, StoreMetricResponse, ToolApprovalListResponse,
        ToolApprovalResponse, TriggerConversationFileExtractionJobRequest, UpdateCampaignRequest,
        UpdateChannelConnectionRequest, UpdateConversationFileExtractionRequest,
        UpdateConversationRequest, UpdateLongTaskRequest, UpdatePackageRequest,
        UpdatePlatformConnectionRequest, UpdateProductAssetRequest, UpdateProductCompetitorRequest,
        UpdateProductKnowledgeRequest, UpdateProductMaterialRequest, UpdateProductRequest,
        UpdateWorkspaceRequest, UpdateWorkspaceRunRequest, UpdateWorkspaceRunTaskRequest,
        UpdateWorkspaceTaskRequest, UpsertDailyBriefingRecordRequest, UpsertPackageRequest,
        WorkspaceListResponse, WorkspaceResponse, WorkspaceRunListResponse, WorkspaceRunResponse,
        WorkspaceRunTaskResponse, WorkspaceTaskListResponse, WorkspaceTaskResponse,
    },
    error::{Error, from_biz_error},
    handlers::sse::build_conversation_history_summary,
    statics::{
        agent_manager::{agent_runtime_llm_usage, run_vision_preprocess, vision_preprocess_config},
        db_manager::get_app_ctx,
        storage_manager::get_storage,
    },
};

const FILE_ACCESS_EXPIRES_SECS: u64 = 600;
const MAX_EXTRACTED_TEXT_CHARS: usize = 100_000;

pub async fn list_conversations(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<ConversationResponse>> {
    let api = ConversationApi::new(get_app_ctx());
    let conversations: Vec<ConversationResponse> = api
        .list_conversations(auth_user.user_id)
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_conversation_response)
        .collect();

    Ok(conversations.into_common_response().to_json())
}

pub async fn create_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateConversationRequest>,
) -> ResponseResult<ConversationDetailResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let detail = api
        .create_conversation(service::dto::conversation::CreateConversationRequest {
            user_id: auth_user.user_id,
            title: req.title,
            first_message: req.first_message,
            agent_id: req.agent_id,
            model_name: None,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_detail_response(detail)
        .into_common_response()
        .to_json())
}

pub async fn get_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
) -> ResponseResult<ConversationDetailResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let detail = api
        .get_conversation(auth_user.user_id, conversation_id)
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_detail_response(detail)
        .into_common_response()
        .to_json())
}

pub async fn add_conversation_message(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<AddConversationMessageRequest>,
) -> ResponseResult<ConversationMessageResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let message = api
        .add_message(service::dto::conversation::AddConversationMessageRequest {
            user_id: auth_user.user_id,
            conversation_id,
            role: req.role,
            content_text: req.content_text,
            content_json: req.content_json,
            strategy_id: req.strategy_id,
            response_mode: req.response_mode,
            routing_mode: req.routing_mode,
            collaboration_mode: req.collaboration_mode,
            learning_level: req.learning_level,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_message_response(message)
        .into_common_response()
        .to_json())
}

pub async fn update_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<UpdateConversationRequest>,
) -> ResponseResult<ConversationResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let conversation = api
        .update_conversation(service::dto::conversation::UpdateConversationRequest {
            user_id: auth_user.user_id,
            conversation_id,
            title: req.title,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_response(conversation)
        .into_common_response()
        .to_json())
}

pub async fn delete_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
) -> ResponseResult<()> {
    let api = ConversationApi::new(get_app_ctx());
    api.delete_conversation(service::dto::conversation::DeleteConversationRequest {
        user_id: auth_user.user_id,
        conversation_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn presign_conversation_file_upload(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<PresignConversationFileUploadRequest>,
) -> ResponseResult<PresignConversationFileUploadResponse> {
    req.validate().map_err(Error::from)?;
    let _ = (auth_user, conversation_id, req);
    Err(
        Error::Custom("conversation file upload signing has moved to file-service".to_string())
            .into(),
    )
}

pub async fn register_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<RegisterConversationFileRequest>,
) -> ResponseResult<ConversationFileResponse> {
    tracing::debug!(
        user_id = %auth_user.user_id,
        conversation_id = %conversation_id,
        object_key = %req.object_key,
        mime_type = ?req.mime_type,
        file_size = ?req.file_size,
        "register conversation file request received"
    );
    req.validate().map_err(Error::from)?;
    let user_id_for_log = auth_user.user_id.clone();
    let user_id_for_extraction = auth_user.user_id.clone();
    let conversation_id_for_log = conversation_id.clone();
    let api = ConversationApi::new(get_app_ctx());
    let file = api
        .register_file(
            service::dto::conversation::RegisterConversationFileRequest {
                user_id: auth_user.user_id,
                conversation_id,
                message_id: req.message_id,
                object_key: req.object_key,
                file_name: req.file_name,
                mime_type: req.mime_type,
                file_size: req.file_size,
                sha256: req.sha256,
            },
        )
        .await
        .map_err(from_biz_error)?;
    tracing::debug!(
        user_id = %user_id_for_log,
        conversation_id = %conversation_id_for_log,
        file_id = %file.id,
        object_key = %file.object_key,
        mime_type = ?file.mime_type,
        status = %file.status,
        "register conversation file succeeded"
    );

    spawn_vision_preprocess_if_needed(&file);
    spawn_document_text_extraction_if_needed(&user_id_for_extraction, &file);

    Ok(map_file_response(file).into_common_response().to_json())
}

pub async fn get_conversation_file_access(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<ConversationFileAccessResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let file = api
        .get_active_file_for_user(auth_user.user_id, file_id)
        .await
        .map_err(from_biz_error)?;
    let signed = get_storage()
        .presign_download(PresignDownloadRequest {
            bucket: BucketKind::Private,
            key: file.object_key.clone(),
            expires_secs: Some(FILE_ACCESS_EXPIRES_SECS),
        })
        .map_err(|err| Error::Custom(err.to_string()))?;

    Ok(ConversationFileAccessResponse {
        file: map_file_response(file),
        method: signed.method,
        download_url: signed.download_url,
        expires_secs: FILE_ACCESS_EXPIRES_SECS,
    }
    .into_common_response()
    .to_json())
}

pub async fn delete_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<()> {
    let api = ConversationApi::new(get_app_ctx());
    api.delete_file(service::dto::conversation::DeleteConversationFileRequest {
        user_id: auth_user.user_id,
        file_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn update_conversation_file_extraction(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
    Json(req): Json<UpdateConversationFileExtractionRequest>,
) -> ResponseResult<ConversationFileResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let file = api
        .update_file_extraction(
            service::dto::conversation::UpdateConversationFileExtractionRequest {
                user_id: auth_user.user_id,
                file_id,
                status: req.status,
                text: req.text,
                content_kind: req.content_kind,
                source: req.source,
                parser: req.parser,
                language: req.language,
                failure_reason: req.failure_reason,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_file_response(file).into_common_response().to_json())
}

pub async fn get_conversation_file_extraction(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<ConversationFileExtractionStatusResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let status = api
        .get_file_extraction_status(auth_user.user_id, file_id)
        .await
        .map_err(from_biz_error)?;
    Ok(status.into_common_response().to_json())
}

pub async fn trigger_conversation_file_extraction_job(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
    Json(req): Json<TriggerConversationFileExtractionJobRequest>,
) -> ResponseResult<ConversationFileExtractionJobResponse> {
    req.validate().map_err(Error::from)?;
    let _force = req.force;
    let job = run_document_text_extraction_task(auth_user.user_id, file_id)
        .await
        .map_err(|err| Error::Custom(format!("document text extraction failed: {err}")))?;
    Ok(job.into_common_response().to_json())
}

pub async fn preview_agent_context(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<AgentContextPreviewRequest>,
) -> ResponseResult<AgentContextPreviewResponse> {
    req.validate().map_err(Error::from)?;
    let user_id = auth_user.user_id;
    let conversation_api = ConversationApi::new(get_app_ctx());
    let conversation_preview = if let Some(conversation_id) = req.conversation_id.as_ref() {
        let conversation = conversation_api
            .get_conversation(user_id.clone(), conversation_id.clone())
            .await
            .map_err(from_biz_error)?;
        let previous_memory = conversation_api
            .get_conversation_memory(user_id.clone(), conversation_id.clone())
            .await
            .map_err(from_biz_error)?;
        Some(build_conversation_history_summary(
            &conversation.messages,
            &req.input,
            previous_memory.as_ref(),
        ))
    } else {
        None
    };
    let workspace_api = WorkspaceApi::new(get_app_ctx());
    let workspace_context = workspace_api
        .context_summary(service::dto::workspace::GetWorkspaceContextSummaryRequest {
            user_id: user_id.clone(),
            workspace_id: req.workspace_id.clone().unwrap_or_default(),
            conversation_id: req.conversation_id.clone(),
            product_id: req.product_id.clone(),
            include_workspace_product_context: true,
            run_id: None,
            context_refs: normalize_preview_context_refs(req.context_refs.clone()),
        })
        .await
        .map_err(from_biz_error)?;
    let workspace_pack = workspace_api
        .context_pack(service::dto::workspace::GetWorkspaceContextPackRequest {
            user_id,
            workspace_id: workspace_context.workspace_id.clone(),
            conversation_id: workspace_context.conversation_id.clone(),
            product_id: workspace_context.product_id.clone(),
            include_workspace_product_context: true,
            run_id: None,
            context_refs: normalize_preview_context_refs(req.context_refs),
            max_items: req.max_items,
            max_chars: req.max_chars,
            query: Some(req.input.clone()),
        })
        .await
        .map_err(from_biz_error)?;
    let included_refs = workspace_pack
        .items
        .iter()
        .filter(|item| item.text.is_some())
        .map(|item| item.ref_key.clone())
        .collect::<Vec<_>>();
    let metadata_only_refs = workspace_pack
        .items
        .iter()
        .filter(|item| item.text.is_none())
        .map(|item| item.ref_key.clone())
        .collect::<Vec<_>>();
    let omitted_refs = workspace_pack.omitted.clone();
    let warnings = workspace_pack.warnings.clone();
    let approximate_chars = workspace_pack
        .items
        .iter()
        .map(|item| item.text_chars)
        .sum::<usize>();
    let history_chars = conversation_preview
        .as_ref()
        .map(|summary| summary.text.chars().count())
        .unwrap_or_default();
    let memory_chars = conversation_preview
        .as_ref()
        .map(|summary| summary.memory_text.chars().count())
        .unwrap_or_default();
    let budget = serde_json::json!({
        "inputChars": req.input.chars().count(),
        "conversationHistoryChars": history_chars,
        "conversationMemoryChars": memory_chars,
        "workspacePackChars": approximate_chars,
        "workspacePackItems": workspace_pack.items.len(),
        "historyMessageCount": conversation_preview
            .as_ref()
            .map(|summary| summary.message_count)
            .unwrap_or_default(),
        "historyTotalMessageCount": conversation_preview
            .as_ref()
            .map(|summary| summary.total_message_count)
            .unwrap_or_default(),
        "historyTruncated": conversation_preview
            .as_ref()
            .map(|summary| summary.truncated)
            .unwrap_or(false),
        "includedRefs": included_refs.len(),
        "metadataOnlyRefs": metadata_only_refs.len(),
        "omittedRefs": omitted_refs.len(),
        "strategy": "recent_history_plus_structured_memory_plus_workspace_pack",
    });
    let conversation_history = conversation_preview.as_ref().map(|summary| {
        serde_json::json!({
            "conversationId": workspace_context.conversation_id.clone(),
            "messageCount": summary.message_count,
            "totalMessageCount": summary.total_message_count,
            "truncated": summary.truncated,
            "relevantOlderCount": summary.relevant_older_count,
            "olderSummary": summary.older_summary.clone(),
            "text": summary.text.clone(),
        })
    });
    let conversation_memory = conversation_preview
        .as_ref()
        .map(|summary| summary.memory_json.clone());

    Ok(AgentContextPreviewResponse {
        workspace_context,
        workspace_pack,
        conversation_history,
        conversation_memory,
        budget,
        included_refs,
        metadata_only_refs,
        omitted_refs,
        warnings,
    }
    .into_common_response()
    .to_json())
}

fn normalize_preview_context_refs(values: Vec<String>) -> Vec<String> {
    let mut refs = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if !refs.iter().any(|item| item == value) {
            refs.push(value.to_string());
        }
    }
    refs
}

fn map_conversation_detail_response(
    detail: service::dto::conversation::ConversationDetailResponse,
) -> ConversationDetailResponse {
    ConversationDetailResponse {
        conversation: map_conversation_response(detail.conversation),
        messages: detail
            .messages
            .into_iter()
            .map(map_message_response)
            .collect(),
        files: detail.files.into_iter().map(map_file_response).collect(),
    }
}

fn map_conversation_response(
    item: service::dto::conversation::ConversationResponse,
) -> ConversationResponse {
    ConversationResponse {
        id: item.id,
        user_id: item.user_id,
        title: item.title,
        status: item.status,
        source: item.source,
        agent_id: item.agent_id,
        model_name: item.model_name,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_message_response(
    item: service::dto::conversation::ConversationMessageResponse,
) -> ConversationMessageResponse {
    ConversationMessageResponse {
        id: item.id,
        conversation_id: item.conversation_id,
        user_id: item.user_id,
        role: item.role,
        content_text: item.content_text,
        content_json: item.content_json,
        status: item.status,
        sequence: item.sequence,
        strategy_id: item.strategy_id,
        response_mode: item.response_mode,
        routing_mode: item.routing_mode,
        collaboration_mode: item.collaboration_mode,
        learning_level: item.learning_level,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_file_response(
    item: service::dto::conversation::ConversationFileResponse,
) -> ConversationFileResponse {
    ConversationFileResponse {
        id: item.id,
        conversation_id: item.conversation_id,
        message_id: item.message_id,
        bucket: item.bucket,
        object_key: item.object_key,
        file_name: item.file_name,
        mime_type: item.mime_type,
        file_size: item.file_size,
        sha256: item.sha256,
        status: item.status,
        metadata: item.metadata,
        created_at: item.created_at,
    }
}

fn spawn_vision_preprocess_if_needed(file: &service::dto::conversation::ConversationFileResponse) {
    let Ok(conversation_file_id) = Uuid::parse_str(&file.id) else {
        tracing::warn!(file_id = %file.id, "vision preprocess skipped: invalid file id");
        return;
    };

    let Ok(agent_config) = vision_preprocess_config() else {
        tracing::warn!("vision preprocess skipped: config unavailable");
        return;
    };
    let Ok(llm_usage) = agent_runtime_llm_usage() else {
        tracing::warn!("vision preprocess skipped: llm usage unavailable");
        return;
    };
    let Ok(llm_name) = agent_config.usage_name(&llm_usage) else {
        tracing::warn!("vision preprocess skipped: configured llm usage unavailable");
        return;
    };

    let config = VisionPreprocessConfig {
        analysis_kind: agent_config.analysis_kind.clone(),
        prompt_key: agent_config.prompt.clone(),
        prompt_version: agent_config.prompt_version.clone(),
        llm_name,
        llm_model: None,
    };
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        conversation_id = %file.conversation_id,
        object_key = %file.object_key,
        mime_type = ?file.mime_type,
        file_size = ?file.file_size,
        analysis_kind = %config.analysis_kind,
        prompt_key = %config.prompt_key,
        prompt_version = %config.prompt_version,
        llm_name = %config.llm_name,
        "vision preprocess task spawned"
    );

    tokio::spawn(async move {
        if let Err(err) = run_vision_preprocess_task(conversation_file_id, config).await {
            tracing::warn!(
                conversation_file_id = %conversation_file_id,
                error = %err,
                "vision preprocess task failed"
            );
        }
    });
}

fn spawn_document_text_extraction_if_needed(
    user_id: &str,
    file: &service::dto::conversation::ConversationFileResponse,
) {
    if is_supported_text_document_mime(file.mime_type.as_deref())
        || file
            .file_name
            .as_deref()
            .is_some_and(is_supported_text_document_name)
        || is_supported_text_document_name(&file.object_key)
    {
        let user_id = user_id.to_string();
        let file_id = file.id.clone();
        tracing::debug!(
            file_id = %file_id,
            object_key = %file.object_key,
            mime_type = ?file.mime_type,
            "document text extraction task spawned"
        );
        tokio::spawn(async move {
            if let Err(err) = run_document_text_extraction_task(user_id, file_id.clone()).await {
                tracing::warn!(
                    file_id = %file_id,
                    error = ?err,
                    "document text extraction task failed"
                );
            }
        });
    }
}

async fn run_document_text_extraction_task(
    user_id: String,
    file_id: String,
) -> Result<ConversationFileExtractionJobResponse, Error> {
    use service::api::storage_object_analysis::{
        CompleteStorageObjectAnalysisRequest, FailStorageObjectAnalysisRequest,
        StorageObjectAnalysisApi,
    };

    let api = ConversationApi::new(get_app_ctx());
    let target = api
        .prepare_file_extraction_target(user_id.clone(), file_id.clone())
        .await
        .map_err(internal_biz_error)?;
    let status = extraction_job_status_from_file(&target.file);
    if target.already_available {
        return Ok(status);
    }

    let analysis_id = Uuid::parse_str(&target.analysis_id)
        .map_err(|err| Error::Custom(format!("invalid analysis id: {err}")))?;
    let analysis_api = StorageObjectAnalysisApi::new(get_app_ctx());
    analysis_api
        .mark_processing(analysis_id, None)
        .await
        .map_err(internal_biz_error)?;

    let result = run_document_text_extraction_io(&target.file).await;
    match result {
        Ok(output) => {
            analysis_api
                .mark_succeeded(CompleteStorageObjectAnalysisRequest {
                    id: analysis_id,
                    llm_model: None,
                    summary: compact_text(&output.text, 2000),
                    result_json: serde_json::json!({
                        "kind": "document_text_extraction",
                        "parser": DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY,
                        "parserVersion": DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION,
                        "fileKind": output.file_kind,
                        "text": output.text,
                        "textChars": output.text_chars,
                        "originalTextChars": output.original_text_chars,
                        "truncated": output.truncated,
                        "checksumSha256": output.checksum_sha256,
                        "storageObjectSha256": target.storage_object_sha256,
                        "sourceName": output.source_name,
                    }),
                })
                .await
                .map_err(internal_biz_error)?;

            let updated = api
                .update_file_extraction(
                    service::dto::conversation::UpdateConversationFileExtractionRequest {
                        user_id,
                        file_id,
                        status: Some("extracted".to_string()),
                        text: Some(format!(
                            "Document text extraction is stored in analysis {}.",
                            analysis_id
                        )),
                        content_kind: Some("document_text_analysis_ref".to_string()),
                        source: Some("server_file_ingest".to_string()),
                        parser: Some(DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY.to_string()),
                        language: None,
                        failure_reason: None,
                        metadata: Some(serde_json::json!({
                            "parserVersion": DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION,
                            "analysisId": analysis_id,
                            "analysisKind": target.analysis_kind,
                            "storageObjectSha256": target.storage_object_sha256,
                            "originalTextChars": output.original_text_chars,
                            "truncated": output.truncated,
                        })),
                    },
                )
                .await
                .map_err(internal_biz_error)?;
            Ok(extraction_job_status_from_file(&updated))
        }
        Err(err) => {
            let error_message = err.to_string();
            analysis_api
                .mark_failed(FailStorageObjectAnalysisRequest {
                    id: analysis_id,
                    error_message: error_message.clone(),
                })
                .await
                .map_err(internal_biz_error)?;

            let updated = api
                .update_file_extraction(
                    service::dto::conversation::UpdateConversationFileExtractionRequest {
                        user_id,
                        file_id,
                        status: Some("failed".to_string()),
                        text: None,
                        content_kind: Some("document_text".to_string()),
                        source: Some("server_file_ingest".to_string()),
                        parser: Some(DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY.to_string()),
                        language: None,
                        failure_reason: Some(error_message),
                        metadata: Some(serde_json::json!({
                            "parserVersion": DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION,
                            "analysisId": analysis_id,
                            "analysisKind": target.analysis_kind,
                            "storageObjectSha256": target.storage_object_sha256,
                        })),
                    },
                )
                .await
                .map_err(internal_biz_error)?;
            Ok(extraction_job_status_from_file(&updated))
        }
    }
}

struct DocumentTextExtractionOutput {
    source_name: String,
    file_kind: String,
    text: String,
    text_chars: usize,
    original_text_chars: usize,
    truncated: bool,
    checksum_sha256: Option<String>,
}

async fn run_document_text_extraction_io(
    file: &service::dto::conversation::ConversationFileResponse,
) -> Result<DocumentTextExtractionOutput, Error> {
    let bytes = get_storage()
        .download_object(DownloadObjectRequest {
            bucket: BucketKind::Private,
            key: file.object_key.clone(),
        })
        .await
        .map_err(|err| Error::Custom(format!("download failed: {err}")))?;
    let source_name = file
        .file_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(file.object_key.as_str())
        .to_string();
    let document = file_ingest::ingest_bytes(&source_name, &bytes)
        .map_err(|err| Error::Custom(format!("parse failed: {err}")))?;
    let text = document.to_ai_text_with_options(&ViewOptions {
        max_table_rows: 100,
        max_cell_chars: 240,
    });
    let text = normalize_required_handler_text(text, "text")?;
    let original_text_chars = text.chars().count();
    let truncated = original_text_chars > MAX_EXTRACTED_TEXT_CHARS;
    let text = if truncated {
        text.chars().take(MAX_EXTRACTED_TEXT_CHARS).collect()
    } else {
        text
    };
    let text_chars = text.chars().count();

    Ok(DocumentTextExtractionOutput {
        source_name,
        file_kind: format!("{:?}", document.kind),
        text,
        text_chars,
        original_text_chars,
        truncated,
        checksum_sha256: document.checksum_sha256,
    })
}

fn extraction_job_status_from_file(
    file: &service::dto::conversation::ConversationFileResponse,
) -> ConversationFileExtractionJobResponse {
    let status = conversation_file_extraction_status_from_metadata(file);
    let message = match status.status.as_str() {
        "extracted" => "document text extraction is stored in storage_object_analyses".to_string(),
        "failed" => status
            .failure_reason
            .clone()
            .unwrap_or_else(|| "document text extraction failed".to_string()),
        "unsupported" => status
            .failure_reason
            .clone()
            .unwrap_or_else(|| "document type is not supported".to_string()),
        _ => "document text extraction is pending".to_string(),
    };

    ConversationFileExtractionJobResponse {
        job_id: format!("document-text-extraction-{}", file.id),
        file_id: status.file_id,
        status: status.status,
        content_kind: status.content_kind,
        parser: status.parser,
        source: status.source,
        message,
        metadata: status.metadata,
    }
}

fn conversation_file_extraction_status_from_metadata(
    file: &service::dto::conversation::ConversationFileResponse,
) -> ConversationFileExtractionStatusResponse {
    let extraction = file
        .metadata
        .get("extraction")
        .and_then(serde_json::Value::as_object);
    let status = file
        .metadata
        .get("extractionStatus")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            extraction.and_then(|value| value.get("status").and_then(serde_json::Value::as_str))
        })
        .map(normalize_file_extraction_status_value)
        .unwrap_or_else(|| "pending".to_string());
    let content_kind = extraction
        .and_then(|value| value.get("contentKind"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let parser = extraction
        .and_then(|value| value.get("parser"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let source = extraction
        .and_then(|value| value.get("source"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let language = extraction
        .and_then(|value| value.get("language"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let failure_reason = extraction
        .and_then(|value| value.get("failureReason"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let text = file_extraction_inline_text(&file.metadata);
    let text_chars = extraction
        .and_then(|value| value.get("textChars"))
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as usize)
        .or_else(|| text.as_deref().map(str::chars).map(Iterator::count))
        .unwrap_or(0);
    let vision_ready = extraction
        .and_then(|value| value.get("visionReady"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_else(|| content_kind.as_deref() == Some("image_vision_input"));
    let readable = status == "extracted"
        && (text_chars > 0
            || content_kind.as_deref() == Some("document_text_analysis_ref")
            || vision_ready);

    ConversationFileExtractionStatusResponse {
        file_id: file.id.clone(),
        status,
        content_kind,
        parser,
        source,
        language,
        summary: text.as_deref().map(|value| compact_text(value, 240)),
        text_chars,
        readable,
        vision_ready,
        failure_reason,
        metadata: file.metadata.clone(),
        updated_at: file.created_at,
    }
}

fn normalize_file_extraction_status_value(value: &str) -> String {
    match value.trim() {
        "ready" | "success" | "extracted" => "extracted".to_string(),
        "error" | "failed" => "failed".to_string(),
        "unsupported" => "unsupported".to_string(),
        "queued" | "processing" | "pending" => "pending".to_string(),
        _ => "pending".to_string(),
    }
}

fn file_extraction_inline_text(metadata: &serde_json::Value) -> Option<String> {
    metadata_text_field(metadata, &["extractedText", "textContent"])
        .or_else(|| metadata_nested_text_field(metadata, "extraction"))
}

fn metadata_nested_text_field(metadata: &serde_json::Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|value| metadata_text_field(value, &["text"]))
}

fn metadata_text_field(metadata: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| metadata.get(*key))
        .filter_map(serde_json::Value::as_str)
        .find_map(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
}

fn normalize_required_handler_text(value: String, label: &str) -> Result<String, Error> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(Error::Custom(format!("{label} cannot be empty")));
    }
    Ok(value)
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn internal_biz_error(err: db_core::error::BizError) -> Error {
    Error::Custom(format!("{err:?}"))
}

fn is_supported_text_document_mime(value: Option<&str>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        mime.as_str(),
        "application/pdf"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel"
            | "text/csv"
            | "text/plain"
            | "text/markdown"
            | "application/json"
    )
}

fn is_supported_text_document_name(value: &str) -> bool {
    let extension = value
        .rsplit_once('.')
        .map(|(_, extension)| extension.trim().to_ascii_lowercase());
    matches!(
        extension.as_deref(),
        Some(
            "pdf"
                | "docx"
                | "xlsx"
                | "xlsm"
                | "xlsb"
                | "xls"
                | "csv"
                | "txt"
                | "md"
                | "markdown"
                | "json"
        )
    )
}

async fn run_vision_preprocess_task(
    conversation_file_id: Uuid,
    config: VisionPreprocessConfig,
) -> Result<(), Error> {
    use service::api::storage_object_analysis::{
        CompleteStorageObjectAnalysisRequest, StorageObjectAnalysisApi,
    };

    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        "vision preprocess task preparing"
    );
    let manager = VisionPreprocessManager::new(get_app_ctx());
    let Some(task) = manager
        .ensure_task_for_conversation_file(EnsureVisionPreprocessRequest {
            conversation_file_id,
            config,
        })
        .await
        .map_err(|err| Error::Custom(format!("vision preprocess task ensure failed: {err:?}")))?
    else {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            "vision preprocess task skipped: unsupported or inactive file"
        );
        return Ok(());
    };
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        storage_object_sha256 = %task.object.sha256,
        analysis_id = %task.analysis.id,
        sha256 = %task.object.sha256,
        current_status = %task.analysis.status,
        "vision preprocess task loaded"
    );

    if !StorageObjectAnalysisApi::can_process(&task.analysis.status) {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            status = %task.analysis.status,
            "vision preprocess task skipped: analysis already processed or running"
        );
        return Ok(());
    }

    let analysis_api = StorageObjectAnalysisApi::new(get_app_ctx());
    analysis_api
        .mark_processing(task.analysis.id, task.analysis.llm_model.clone())
        .await
        .map_err(|err| {
            Error::Custom(format!("vision preprocess mark processing failed: {err:?}"))
        })?;
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        analysis_id = %task.analysis.id,
        "vision preprocess marked processing"
    );

    let mime_type = task
        .file
        .mime_type
        .clone()
        .unwrap_or_else(|| "image/jpeg".to_string());
    let result = async {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            object_key = %task.file.object_key,
            bucket = %task.file.bucket,
            "vision preprocess image download starting"
        );
        let bytes = get_storage()
            .download_object(DownloadObjectRequest {
                bucket: BucketKind::Private,
                key: task.file.object_key.clone(),
            })
            .await
            .map_err(|err| Error::Custom(format!("vision preprocess download failed: {err}")))?;
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            original_bytes = bytes.len(),
            mime_type = %mime_type,
            "vision preprocess image download finished"
        );
        let data_url = vision_image_data_url(&mime_type, &bytes)
            .map_err(|err| Error::Custom(format!("vision preprocess data url failed: {err}")))?;
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            data_url_bytes = data_url.len(),
            "vision preprocess image prepared"
        );
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            "vision preprocess agent starting"
        );
        run_vision_preprocess(agent::api::VisionPreprocessInput {
            data_url,
            file_name: task.file.file_name.clone(),
            mime_type: task.file.mime_type.clone(),
            object_key: task.file.object_key.clone(),
            bucket: task.file.bucket.clone(),
            file_size: task.file.file_size,
        })
        .await
    }
    .await;

    match result {
        Ok(output) => {
            analysis_api
                .mark_succeeded(CompleteStorageObjectAnalysisRequest {
                    id: task.analysis.id,
                    llm_model: output.llm_model,
                    summary: output.summary,
                    result_json: output.result_json,
                })
                .await
                .map_err(|err| {
                    Error::Custom(format!("vision preprocess mark succeeded failed: {err:?}"))
                })?;
            tracing::debug!(
                conversation_file_id = %conversation_file_id,
                analysis_id = %task.analysis.id,
                "vision preprocess task succeeded"
            );
        }
        Err(err) => {
            analysis_api
                .delete(task.analysis.id)
                .await
                .map_err(|delete_err| {
                    Error::Custom(format!(
                        "vision preprocess delete failed analysis failed: {delete_err:?}"
                    ))
                })?;
            return Err(err);
        }
    }

    Ok(())
}
