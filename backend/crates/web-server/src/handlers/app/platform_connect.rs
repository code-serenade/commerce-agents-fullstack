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
        PlatformConnectionResponse, PlatformExecutionAdapterCatalogResponse,
        PlatformExecutionPreflightRequest, PlatformExecutionPreflightResponse,
        PlatformSummaryResponse, PlatformSyncStatusListResponse,
        PresignConversationFileUploadRequest, PresignConversationFileUploadResponse,
        PresignProductAssetUploadRequest, PresignProductAssetUploadResponse,
        ProductAssetAccessResponse, ProductAssetResponse, ProductCompetitorResponse,
        ProductDashboardResponse, ProductDetailResponse, ProductKnowledgeResponse,
        ProductMaterialResponse, ProductResponse, RegisterAppUserRequest,
        RegisterConversationFileRequest, RegisterProductAssetRequest, SaveChannelConnectionRequest,
        SavePackageLocksRequest, SavePlatformConnectionRequest, SaveStoreMetricRequest,
        SnoozeAlertRequest, StoreMetricListResponse, StoreMetricResponse, ToolApprovalListResponse,
        ToolApprovalResponse, UpdateCampaignRequest, UpdateChannelConnectionRequest,
        UpdateConversationRequest, UpdateLongTaskRequest, UpdatePackageRequest,
        UpdatePlatformConnectionRequest, UpdateProductAssetRequest, UpdateProductCompetitorRequest,
        UpdateProductKnowledgeRequest, UpdateProductMaterialRequest, UpdateProductRequest,
        UpdateWorkspaceRequest, UpdateWorkspaceRunRequest, UpdateWorkspaceRunTaskRequest,
        UpdateWorkspaceTaskRequest, UpsertDailyBriefingRecordRequest, UpsertPackageRequest,
        WorkspaceListResponse, WorkspaceResponse, WorkspaceRunListResponse, WorkspaceRunResponse,
        WorkspaceRunTaskResponse, WorkspaceTaskListResponse, WorkspaceTaskResponse,
    },
    error::{Error, from_biz_error},
    statics::{db_manager::get_app_ctx, storage_manager::get_storage},
};

pub async fn list_platform_catalog() -> ResponseResult<PlatformCatalogResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    Ok(api.platform_catalog().into_common_response().to_json())
}

pub async fn list_platform_execution_adapters(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<PlatformExecutionAdapterCatalogResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .execution_adapter_catalog(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn platform_execution_preflight(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<PlatformExecutionPreflightRequest>,
) -> ResponseResult<PlatformExecutionPreflightResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .execution_preflight(
            service::dto::platform_connect::PlatformExecutionPreflightRequest {
                user_id: auth_user.user_id,
                skill_name: req.skill_name,
                args: req.args,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_platform_connections(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListPlatformConnectionsQuery>,
) -> ResponseResult<PlatformConnectionListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_platform_connections(
            service::dto::platform_connect::ListPlatformConnectionsRequest {
                user_id: auth_user.user_id,
                platform: req.platform,
                enabled: req.enabled,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SavePlatformConnectionRequest>,
) -> ResponseResult<PlatformConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_platform_connection(
            service::dto::platform_connect::SavePlatformConnectionRequest {
                user_id: auth_user.user_id,
                platform: req.platform,
                app_key: req.app_key,
                app_secret: req.app_secret,
                access_token: req.access_token,
                refresh_token: req.refresh_token,
                token_expires_at: req.token_expires_at,
                shop_id: req.shop_id,
                shop_name: req.shop_name,
                enabled: req.enabled,
                test_result: req.test_result,
                last_synced_at: req.last_synced_at,
                sync_status: req.sync_status,
                sync_message: req.sync_message,
                extra: req.extra,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<PlatformConnectionResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .get_platform_connection(
            service::dto::platform_connect::GetPlatformConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
    Json(req): Json<UpdatePlatformConnectionRequest>,
) -> ResponseResult<PlatformConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .update_platform_connection(
            service::dto::platform_connect::UpdatePlatformConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
                platform: req.platform,
                app_key: req.app_key,
                app_secret: req.app_secret,
                access_token: req.access_token,
                refresh_token: req.refresh_token,
                token_expires_at: req.token_expires_at,
                shop_id: req.shop_id,
                shop_name: req.shop_name,
                enabled: req.enabled,
                test_result: req.test_result,
                last_synced_at: req.last_synced_at,
                sync_status: req.sync_status,
                sync_message: req.sync_message,
                extra: req.extra,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn disable_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<()> {
    let api = PlatformConnectApi::new(get_app_ctx());
    api.disable_platform_connection(
        service::dto::platform_connect::DisablePlatformConnectionRequest {
            user_id: auth_user.user_id,
            connection_id,
        },
    )
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_channel_catalog() -> ResponseResult<ChannelCatalogResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    Ok(api.channel_catalog().into_common_response().to_json())
}

pub async fn list_channel_connections(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListChannelConnectionsQuery>,
) -> ResponseResult<ChannelConnectionListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_channel_connections(
            service::dto::platform_connect::ListChannelConnectionsRequest {
                user_id: auth_user.user_id,
                channel: req.channel,
                enabled: req.enabled,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SaveChannelConnectionRequest>,
) -> ResponseResult<ChannelConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_channel_connection(
            service::dto::platform_connect::SaveChannelConnectionRequest {
                user_id: auth_user.user_id,
                channel: req.channel,
                webhook_url: req.webhook_url,
                bot_token: req.bot_token,
                chat_id: req.chat_id,
                secret: req.secret,
                extra_credentials: req.extra_credentials,
                enabled: req.enabled,
                last_error: req.last_error,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<ChannelConnectionResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .get_channel_connection(
            service::dto::platform_connect::GetChannelConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
    Json(req): Json<UpdateChannelConnectionRequest>,
) -> ResponseResult<ChannelConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .update_channel_connection(
            service::dto::platform_connect::UpdateChannelConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
                channel: req.channel,
                webhook_url: req.webhook_url,
                bot_token: req.bot_token,
                chat_id: req.chat_id,
                secret: req.secret,
                extra_credentials: req.extra_credentials,
                enabled: req.enabled,
                last_error: req.last_error,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn disable_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<()> {
    let api = PlatformConnectApi::new(get_app_ctx());
    api.disable_channel_connection(
        service::dto::platform_connect::DisableChannelConnectionRequest {
            user_id: auth_user.user_id,
            connection_id,
        },
    )
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_store_metrics(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListStoreMetricsQuery>,
) -> ResponseResult<StoreMetricListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_metrics(service::dto::platform_connect::ListStoreMetricsRequest {
            user_id: auth_user.user_id,
            platform: req.platform,
            metric_key: req.metric_key,
            metric_date_from: req.metric_date_from,
            metric_date_to: req.metric_date_to,
            source: req.source,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_store_metric(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SaveStoreMetricRequest>,
) -> ResponseResult<StoreMetricResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_metric(service::dto::platform_connect::SaveStoreMetricRequest {
            user_id: auth_user.user_id,
            platform: req.platform,
            metric_date: req.metric_date,
            metric_key: req.metric_key,
            metric_value: req.metric_value,
            dimension_key: req.dimension_key,
            dimension_value: req.dimension_value,
            source: req.source,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn platform_sync_status(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<PlatformSyncStatusListResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .sync_status(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn platform_summary(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<PlatformSummaryResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .summary(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}
