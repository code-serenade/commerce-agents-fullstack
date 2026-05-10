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
        SnoozeAlertRequest, StoreMetricListResponse, StoreMetricResponse,
        SyncLegacyBuiltinPackagesRequest, SyncLegacyBuiltinPackagesResponse,
        ToolApprovalListResponse, ToolApprovalResponse, UpdateCampaignRequest,
        UpdateChannelConnectionRequest, UpdateConversationRequest, UpdateLongTaskRequest,
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

pub async fn list_package_catalog(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListPackageCatalogQuery>,
) -> ResponseResult<PackageCatalogResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .catalog(
            service::dto::package_governance::ListPackageCatalogRequest {
                user_id: auth_user.user_id,
                package_type: req.package_type,
                status: req.status,
                include_disabled: req.include_disabled,
                keyword: req.keyword,
                page: req.page,
                page_size: req.page_size,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn upsert_package(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpsertPackageRequest>,
) -> ResponseResult<PackageResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .upsert_package(service::dto::package_governance::UpsertPackageRequest {
            user_id: auth_user.user_id,
            package_type: req.package_type,
            package_id: req.package_id,
            package_name: req.package_name,
            version: req.version,
            status: req.status,
            manifest_path: req.manifest_path,
            dependencies: req.dependencies,
            compatibility: req.compatibility,
            metadata: req.metadata,
            reason: req.reason,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn sync_legacy_builtin_packages(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SyncLegacyBuiltinPackagesRequest>,
) -> ResponseResult<SyncLegacyBuiltinPackagesResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .sync_legacy_builtin_packages(
            service::dto::package_governance::SyncLegacyBuiltinPackagesRequest {
                user_id: auth_user.user_id,
                reset_status: req.reset_status,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn get_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
) -> ResponseResult<PackageResponse> {
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .get_package(service::dto::package_governance::GetPackageRequest {
            user_id: auth_user.user_id,
            package_type,
            package_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn update_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
    Json(req): Json<UpdatePackageRequest>,
) -> ResponseResult<PackageResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .update_package(service::dto::package_governance::UpdatePackageRequest {
            user_id: auth_user.user_id,
            package_type,
            package_id,
            package_name: req.package_name,
            version: req.version,
            status: req.status,
            manifest_path: req.manifest_path,
            dependencies: req.dependencies,
            compatibility: req.compatibility,
            metadata: req.metadata,
            reason: req.reason,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn validate_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
) -> ResponseResult<PackageValidationResponse> {
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .validate_package(service::dto::package_governance::GetPackageRequest {
            user_id: auth_user.user_id,
            package_type,
            package_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn activate_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
    Json(req): Json<PackageStatusChangeRequest>,
) -> ResponseResult<PackageResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .activate_package(
            service::dto::package_governance::PackageStatusChangeRequest {
                user_id: auth_user.user_id,
                package_type,
                package_id,
                reason: req.reason,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn deactivate_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
    Json(req): Json<PackageStatusChangeRequest>,
) -> ResponseResult<PackageResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .deactivate_package(
            service::dto::package_governance::PackageStatusChangeRequest {
                user_id: auth_user.user_id,
                package_type,
                package_id,
                reason: req.reason,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn rollback_package(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
    Json(req): Json<PackageStatusChangeRequest>,
) -> ResponseResult<PackageResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .rollback_package(
            service::dto::package_governance::PackageStatusChangeRequest {
                user_id: auth_user.user_id,
                package_type,
                package_id,
                reason: req.reason,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn list_package_history(
    Extension(auth_user): Extension<AuthUser>,
    Path((package_type, package_id)): Path<(String, String)>,
    Query(req): Query<ListPackageHistoryQuery>,
) -> ResponseResult<PackageHistoryListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .package_history(
            service::dto::package_governance::ListPackageHistoryRequest {
                user_id: auth_user.user_id,
                package_type,
                package_id,
                limit: req.limit,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn list_package_locks(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
) -> ResponseResult<PackageLockListResponse> {
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .list_package_locks(service::dto::package_governance::ListPackageLocksRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn save_package_locks(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Json(req): Json<SavePackageLocksRequest>,
) -> ResponseResult<PackageLockListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PackageGovernanceApi::new(get_app_ctx());
    let entries = req
        .entries
        .into_iter()
        .map(|entry| service::dto::package_governance::PackageLockInput {
            package_type: entry.package_type,
            package_id: entry.package_id,
            package_name: entry.package_name,
            locked_version: entry.locked_version,
            locked_status: entry.locked_status,
            required: entry.required,
            lock_source: entry.lock_source,
            manifest_path: entry.manifest_path,
            notes: entry.notes,
            metadata: entry.metadata,
        })
        .collect();
    let response = api
        .save_package_locks(service::dto::package_governance::SavePackageLocksRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            entries,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn package_lock_drift(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
) -> ResponseResult<PackageLockDriftResponse> {
    let api = PackageGovernanceApi::new(get_app_ctx());
    let response = api
        .package_lock_drift(service::dto::package_governance::PackageLockDriftRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}
