#![allow(unused_imports)]

use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use service::api::{
    app::AppApi,
    approval_audit::ApprovalAuditApi,
    briefing_ops::BriefingOpsApi,
    campaign::CampaignApi,
    conversation::ConversationApi,
    package_governance::PackageGovernanceApi,
    platform_connect::PlatformConnectApi,
    product::ProductApi,
    product_asset::{ProductAssetApi, product_asset_object_key},
    product_competitor::ProductCompetitorApi,
    product_knowledge::ProductKnowledgeApi,
    product_material::ProductMaterialApi,
    workspace::WorkspaceApi,
};
use storage::{BucketKind, FileKind, PresignDownloadRequest, PresignUploadRequest};
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
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
    statics::{db_manager::get_app_ctx, storage_manager::get_storage},
};

const FILE_UPLOAD_EXPIRES_SECS: u64 = 600;

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
    let api = ConversationApi::new(get_app_ctx());
    api.get_conversation(auth_user.user_id.clone(), conversation_id.clone())
        .await
        .map_err(from_biz_error)?;

    let storage = get_storage();
    let ext = FileKind::ChatAttachment
        .normalize_ext(&req.extension)
        .map_err(|err| Error::Custom(err.to_string()))?;
    let content_type = req.content_type.or_else(|| {
        Some(
            FileKind::ChatAttachment
                .content_type_for_ext(&ext)
                .to_string(),
        )
    });
    let user_scope = storage.user_scope_key(&auth_user.user_id);
    let object_key = storage::key::chat_attachment_key(&user_scope, &conversation_id, &ext)
        .map_err(|err| Error::Custom(err.to_string()))?;
    let signed = storage
        .presign_upload(PresignUploadRequest {
            bucket: BucketKind::Private,
            key: object_key,
            content_type,
            expires_secs: Some(FILE_UPLOAD_EXPIRES_SECS),
        })
        .map_err(|err| Error::Custom(err.to_string()))?;

    Ok(PresignConversationFileUploadResponse {
        method: signed.method,
        upload_url: signed.upload_url,
        bucket: signed.bucket,
        object_key: signed.key,
        content_type: signed.content_type,
        expires_secs: signed.expires_secs,
    }
    .into_common_response()
    .to_json())
}

pub async fn register_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<RegisterConversationFileRequest>,
) -> ResponseResult<ConversationFileResponse> {
    req.validate().map_err(Error::from)?;
    let storage = get_storage();
    let user_scope = storage.user_scope_key(&auth_user.user_id);
    let expected_prefix = format!("chat/{user_scope}/{conversation_id}/");
    if !req.object_key.starts_with(&expected_prefix) {
        return Err(Error::Custom("invalid conversation file object key".to_string()).into());
    }
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

    Ok(map_file_response(file).into_common_response().to_json())
}

pub async fn get_conversation_file_access(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<ConversationFileAccessResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let access = api
        .file_access(get_storage(), auth_user.user_id, file_id)
        .await
        .map_err(from_biz_error)?;

    Ok(ConversationFileAccessResponse {
        file: map_file_response(access.file),
        method: access.method,
        download_url: access.download_url,
        expires_secs: access.expires_secs,
    }
    .into_common_response()
    .to_json())
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
    let api = ConversationApi::new(get_app_ctx());
    let job = api
        .trigger_file_extraction_job(
            get_storage(),
            service::dto::conversation::TriggerConversationFileExtractionJobRequest {
                user_id: auth_user.user_id,
                file_id,
                mode: req.mode,
                force: req.force,
            },
        )
        .await
        .map_err(from_biz_error)?;
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
