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

pub async fn list_long_tasks(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListLongTasksQuery>,
) -> ResponseResult<LongTaskListResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .list_long_tasks(service::dto::briefing_ops::ListLongTasksRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            task_type: req.task_type,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .create_long_task(service::dto::briefing_ops::CreateLongTaskRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            task_type: req.task_type,
            title: req.title,
            status: req.status,
            progress: req.progress,
            total_steps: req.total_steps,
            current_step: req.current_step,
            role: req.role,
            input_params: req.input_params,
            result: req.result,
            error_message: req.error_message,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
) -> ResponseResult<LongTaskResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .get_long_task(service::dto::briefing_ops::GetLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
    Json(req): Json<UpdateLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .update_long_task(service::dto::briefing_ops::UpdateLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
            status: req.status,
            title: req.title,
            progress: req.progress,
            total_steps: req.total_steps,
            current_step: req.current_step,
            role: req.role,
            result: req.result,
            error_message: req.error_message,
            started_at: req.started_at,
            completed_at: req.completed_at,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn cancel_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
    Json(req): Json<CancelLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .cancel_long_task(service::dto::briefing_ops::CancelLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
            error_message: req.error_message,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}
