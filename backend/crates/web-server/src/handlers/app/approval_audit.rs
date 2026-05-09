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

pub async fn list_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListExecutionQueueQuery>,
) -> ResponseResult<ExecutionQueueListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_execution_queue(service::dto::workspace::ListExecutionQueueRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            action_type: req.action_type,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_execution_queue(service::dto::workspace::CreateExecutionQueueRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            workspace_task_id: req.workspace_task_id,
            action_type: req.action_type,
            description: req.description,
            platform: req.platform,
            payload: req.payload,
            expected_impact: req.expected_impact,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
) -> ResponseResult<ExecutionQueueResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_execution_queue(service::dto::workspace::GetExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn approve_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
    Json(req): Json<DecideExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .approve_execution_queue(service::dto::workspace::DecideExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn reject_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
    Json(req): Json<DecideExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .reject_execution_queue(service::dto::workspace::DecideExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn list_tool_approvals(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListToolApprovalsQuery>,
) -> ResponseResult<ToolApprovalListResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .list_tool_approvals(service::dto::approval_audit::ListToolApprovalsRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            skill_name: req.skill_name,
            risk_level: req.risk_level,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .create_tool_approval(service::dto::approval_audit::CreateToolApprovalRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            conversation_id: req.conversation_id,
            request_role: req.request_role,
            skill_name: req.skill_name,
            tool_call_id: req.tool_call_id,
            risk_level: req.risk_level,
            args: req.args,
            action_key: req.action_key,
            reason: req.reason,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
) -> ResponseResult<ToolApprovalResponse> {
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .get_tool_approval(service::dto::approval_audit::GetToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn approve_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
    Json(req): Json<DecideToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .approve_tool_approval(service::dto::approval_audit::DecideToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn reject_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
    Json(req): Json<DecideToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .reject_tool_approval(service::dto::approval_audit::DecideToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_audit_logs(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListAuditLogsQuery>,
) -> ResponseResult<AuditLogListResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .list_audit_logs(service::dto::approval_audit::ListAuditLogsRequest {
            user_id: auth_user.user_id,
            actor_user_id: req.actor_user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            execution_queue_id: req.execution_queue_id,
            tool_approval_id: req.tool_approval_id,
            action: req.action,
            target_type: req.target_type,
            status: req.status,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_audit_log(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAuditLogRequest>,
) -> ResponseResult<AuditLogResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .create_audit_log(service::dto::approval_audit::CreateAuditLogRequest {
            user_id: auth_user.user_id,
            actor_user_id: req.actor_user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            execution_queue_id: req.execution_queue_id,
            tool_approval_id: req.tool_approval_id,
            action: req.action,
            target_type: req.target_type,
            target_id: req.target_id,
            status: req.status,
            reason: req.reason,
            request_id: req.request_id,
            trace_id: req.trace_id,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

fn map_execution_queue_list_response(
    item: service::dto::workspace::ExecutionQueueListResponse,
) -> ExecutionQueueListResponse {
    ExecutionQueueListResponse {
        items: item
            .items
            .into_iter()
            .map(map_execution_queue_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_execution_queue_response(
    item: service::dto::workspace::ExecutionQueueResponse,
) -> ExecutionQueueResponse {
    ExecutionQueueResponse {
        id: item.id,
        user_id: item.user_id,
        workspace_id: item.workspace_id,
        run_id: item.run_id,
        workspace_task_id: item.workspace_task_id,
        action_type: item.action_type,
        description: item.description,
        platform: item.platform,
        payload: item.payload,
        expected_impact: item.expected_impact,
        status: item.status,
        decision_reason: item.decision_reason,
        result: item.result,
        created_at: item.created_at,
        decided_at: item.decided_at,
    }
}
