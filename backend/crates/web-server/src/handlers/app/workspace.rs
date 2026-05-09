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

pub async fn list_workspaces(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListWorkspacesQuery>,
) -> ResponseResult<WorkspaceListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_workspaces(service::dto::workspace::ListWorkspacesRequest {
            user_id: auth_user.user_id,
            status: req.status,
            product_id: req.product_id,
            campaign_id: req.campaign_id,
            workspace_type: req.workspace_type,
            keyword: req.keyword,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_workspace(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> ResponseResult<WorkspaceResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_workspace(service::dto::workspace::CreateWorkspaceRequest {
            user_id: auth_user.user_id,
            product_id: req.product_id,
            campaign_id: req.campaign_id,
            title: req.title,
            workspace_type: req.workspace_type,
            config: req.config,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_workspace(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
) -> ResponseResult<WorkspaceResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_workspace(service::dto::workspace::GetWorkspaceRequest {
            user_id: auth_user.user_id,
            workspace_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_response(response)
        .into_common_response()
        .to_json())
}

pub async fn update_workspace(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> ResponseResult<WorkspaceResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .update_workspace(service::dto::workspace::UpdateWorkspaceRequest {
            user_id: auth_user.user_id,
            workspace_id,
            product_id: req.product_id,
            campaign_id: req.campaign_id,
            title: req.title,
            workspace_type: req.workspace_type,
            status: req.status,
            config: req.config,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_response(response)
        .into_common_response()
        .to_json())
}

pub async fn delete_workspace(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
) -> ResponseResult<()> {
    let api = WorkspaceApi::new(get_app_ctx());
    api.delete_workspace(service::dto::workspace::DeleteWorkspaceRequest {
        user_id: auth_user.user_id,
        workspace_id,
    })
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_workspace_tasks(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<ListWorkspaceTasksQuery>,
) -> ResponseResult<WorkspaceTaskListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_tasks(service::dto::workspace::ListWorkspaceTasksRequest {
            user_id: auth_user.user_id,
            workspace_id,
            status: req.status,
            owner_role: req.owner_role,
            keyword: req.keyword,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_task_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_workspace_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Json(req): Json<CreateWorkspaceTaskRequest>,
) -> ResponseResult<WorkspaceTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_task(service::dto::workspace::CreateWorkspaceTaskRequest {
            user_id: auth_user.user_id,
            workspace_id,
            title: req.title,
            description: req.description,
            owner_role: req.owner_role,
            status: req.status,
            priority: req.priority,
            depends_on: req.depends_on,
            acceptance_criteria: req.acceptance_criteria,
            result: req.result,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_task_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_workspace_task(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, task_id)): Path<(String, String)>,
) -> ResponseResult<WorkspaceTaskResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_task(service::dto::workspace::GetWorkspaceTaskRequest {
            user_id: auth_user.user_id,
            workspace_id,
            task_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_task_response(response)
        .into_common_response()
        .to_json())
}

pub async fn update_workspace_task(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, task_id)): Path<(String, String)>,
    Json(req): Json<UpdateWorkspaceTaskRequest>,
) -> ResponseResult<WorkspaceTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .update_task(service::dto::workspace::UpdateWorkspaceTaskRequest {
            user_id: auth_user.user_id,
            workspace_id,
            task_id,
            title: req.title,
            description: req.description,
            owner_role: req.owner_role,
            status: req.status,
            priority: req.priority,
            depends_on: req.depends_on,
            acceptance_criteria: req.acceptance_criteria,
            result: req.result,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_task_response(response)
        .into_common_response()
        .to_json())
}

pub async fn delete_workspace_task(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, task_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = WorkspaceApi::new(get_app_ctx());
    api.delete_task(service::dto::workspace::DeleteWorkspaceTaskRequest {
        user_id: auth_user.user_id,
        workspace_id,
        task_id,
    })
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_workspace_runs(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<ListWorkspaceRunsQuery>,
) -> ResponseResult<WorkspaceRunListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_runs(service::dto::workspace::ListWorkspaceRunsRequest {
            user_id: auth_user.user_id,
            workspace_id,
            status: req.status,
            source: req.source,
            keyword: req.keyword,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_workspace_run(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Json(req): Json<CreateWorkspaceRunRequest>,
) -> ResponseResult<WorkspaceRunResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_run(service::dto::workspace::CreateWorkspaceRunRequest {
            user_id: auth_user.user_id,
            workspace_id,
            goal: req.goal,
            requirements: req.requirements,
            status: req.status,
            source: req.source,
            summary: req.summary,
            report: req.report,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_workspace_run(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
) -> ResponseResult<WorkspaceRunResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_run(service::dto::workspace::GetWorkspaceRunRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_response(response)
        .into_common_response()
        .to_json())
}

pub async fn update_workspace_run(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Json(req): Json<UpdateWorkspaceRunRequest>,
) -> ResponseResult<WorkspaceRunResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .update_run(service::dto::workspace::UpdateWorkspaceRunRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            goal: req.goal,
            requirements: req.requirements,
            status: req.status,
            source: req.source,
            summary: req.summary,
            report: req.report,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_response(response)
        .into_common_response()
        .to_json())
}

pub async fn list_workspace_run_tasks(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Query(req): Query<ListWorkspaceRunTasksQuery>,
) -> ResponseResult<Vec<WorkspaceRunTaskResponse>> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_run_tasks(service::dto::workspace::ListWorkspaceRunTasksRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            status: req.status,
        })
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_workspace_run_task_response)
        .collect::<Vec<_>>();
    Ok(response.into_common_response().to_json())
}

pub async fn create_workspace_run_task(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Json(req): Json<CreateWorkspaceRunTaskRequest>,
) -> ResponseResult<WorkspaceRunTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_run_task(service::dto::workspace::CreateWorkspaceRunTaskRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            workspace_task_id: req.workspace_task_id,
            seq: req.seq,
            owner_role: req.owner_role,
            status: req.status,
            attempts: req.attempts,
            acceptance_score: req.acceptance_score,
            acceptance_passed: req.acceptance_passed,
            acceptance_notes: req.acceptance_notes,
            execution_notes: req.execution_notes,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_task_response(response)
        .into_common_response()
        .to_json())
}

pub async fn update_workspace_run_task(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id, run_task_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateWorkspaceRunTaskRequest>,
) -> ResponseResult<WorkspaceRunTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .update_run_task(service::dto::workspace::UpdateWorkspaceRunTaskRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            run_task_id,
            seq: req.seq,
            owner_role: req.owner_role,
            status: req.status,
            attempts: req.attempts,
            acceptance_score: req.acceptance_score,
            acceptance_passed: req.acceptance_passed,
            acceptance_notes: req.acceptance_notes,
            execution_notes: req.execution_notes,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_run_task_response(response)
        .into_common_response()
        .to_json())
}

fn map_workspace_list_response(
    item: service::dto::workspace::WorkspaceListResponse,
) -> WorkspaceListResponse {
    WorkspaceListResponse {
        workspaces: item
            .workspaces
            .into_iter()
            .map(map_workspace_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_workspace_response(item: service::dto::workspace::WorkspaceResponse) -> WorkspaceResponse {
    WorkspaceResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        campaign_id: item.campaign_id,
        title: item.title,
        workspace_type: item.workspace_type,
        status: item.status,
        config: item.config,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_workspace_task_list_response(
    item: service::dto::workspace::WorkspaceTaskListResponse,
) -> WorkspaceTaskListResponse {
    WorkspaceTaskListResponse {
        tasks: item
            .tasks
            .into_iter()
            .map(map_workspace_task_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_workspace_task_response(
    item: service::dto::workspace::WorkspaceTaskResponse,
) -> WorkspaceTaskResponse {
    WorkspaceTaskResponse {
        id: item.id,
        workspace_id: item.workspace_id,
        user_id: item.user_id,
        title: item.title,
        description: item.description,
        owner_role: item.owner_role,
        status: item.status,
        priority: item.priority,
        depends_on: item.depends_on,
        acceptance_criteria: item.acceptance_criteria,
        result: item.result,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_workspace_run_list_response(
    item: service::dto::workspace::WorkspaceRunListResponse,
) -> WorkspaceRunListResponse {
    WorkspaceRunListResponse {
        runs: item
            .runs
            .into_iter()
            .map(map_workspace_run_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_workspace_run_response(
    item: service::dto::workspace::WorkspaceRunResponse,
) -> WorkspaceRunResponse {
    WorkspaceRunResponse {
        id: item.id,
        workspace_id: item.workspace_id,
        user_id: item.user_id,
        goal: item.goal,
        requirements: item.requirements,
        status: item.status,
        source: item.source,
        summary: item.summary,
        report: item.report,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_workspace_run_task_response(
    item: service::dto::workspace::WorkspaceRunTaskResponse,
) -> WorkspaceRunTaskResponse {
    WorkspaceRunTaskResponse {
        id: item.id,
        run_id: item.run_id,
        workspace_id: item.workspace_id,
        workspace_task_id: item.workspace_task_id,
        user_id: item.user_id,
        seq: item.seq,
        owner_role: item.owner_role,
        status: item.status,
        attempts: item.attempts,
        acceptance_score: item.acceptance_score,
        acceptance_passed: item.acceptance_passed,
        acceptance_notes: item.acceptance_notes,
        execution_notes: item.execution_notes,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
