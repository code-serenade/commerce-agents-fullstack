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
    storage_resource_manager::{CleanupOrphanedStorageObjectsRequest, StorageResourceManager},
    workspace::WorkspaceApi,
};
use storage::PresignDownloadRequest;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::app::{
        ActAlertRequest, AddConversationMessageRequest, AlertListResponse, AlertResponse,
        AlertSummaryResponse, AppUserResponse, AuditLogListResponse, AuditLogResponse,
        BackgroundEventListResponse, BackgroundEventResponse, BatchUpdateResponse,
        CampaignListResponse, CampaignResponse, CancelLongTaskRequest, ChannelCatalogResponse,
        ChannelConnectionListResponse, ChannelConnectionResponse, CompetitorChangeResponse,
        ConsumeBackgroundEventsRequest, ConversationDetailResponse, ConversationFileAccessResponse,
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
        ToolApprovalResponse, UpdateCampaignRequest, UpdateChannelConnectionRequest,
        UpdateConversationFileExtractionRequest, UpdateConversationRequest, UpdateLongTaskRequest,
        UpdatePackageRequest, UpdatePlatformConnectionRequest, UpdateProductAssetRequest,
        UpdateProductCompetitorRequest, UpdateProductKnowledgeRequest,
        UpdateProductMaterialRequest, UpdateProductRequest, UpdateWorkspaceRequest,
        UpdateWorkspaceRunRequest, UpdateWorkspaceRunTaskRequest, UpdateWorkspaceTaskRequest,
        UpsertDailyBriefingRecordRequest, UpsertPackageRequest, WorkspaceListResponse,
        WorkspaceResponse, WorkspaceRunListResponse, WorkspaceRunResponse,
        WorkspaceRunTaskResponse, WorkspaceTaskListResponse, WorkspaceTaskResponse,
    },
    error::{Error, from_biz_error},
    statics::{db_manager::get_app_ctx, storage_manager::get_storage},
};

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
    StorageResourceManager::new(get_app_ctx())
        .cleanup_orphaned_objects(
            get_storage(),
            CleanupOrphanedStorageObjectsRequest { limit: Some(50) },
        )
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
    req.validate().map_err(Error::from)?;
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

pub async fn delete_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<()> {
    let api = ConversationApi::new(get_app_ctx());
    api.delete_file(
        get_storage(),
        service::dto::conversation::DeleteConversationFileRequest {
            user_id: auth_user.user_id,
            file_id,
        },
    )
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
