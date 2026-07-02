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
        UpdateConversationRequest, UpdateCurrentAppUserProfileRequest, UpdateLongTaskRequest,
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

fn map_role_response(role: service::dto::app::RoleResponse) -> crate::dto::app::RoleResponse {
    crate::dto::app::RoleResponse {
        id: role.id,
        name: role.name,
        code: role.code,
        created_at: role.created_at,
    }
}

fn map_app_user_response(app_user: service::dto::app::AppUserResponse) -> AppUserResponse {
    AppUserResponse {
        user_id: app_user.user_id,
        display_id: app_user.display_id,
        display_name: app_user.display_name,
        school_name: app_user.school_name,
        team_name: app_user.team_name,
        remark: app_user.remark,
        status: match app_user.status {
            service::dto::app::AppUserStatus::Enabled => crate::dto::app::AppUserStatus::Enabled,
            service::dto::app::AppUserStatus::Disabled => crate::dto::app::AppUserStatus::Disabled,
        },
        created_at: app_user.created_at,
        updated_at: app_user.updated_at,
        roles: app_user.roles.into_iter().map(map_role_response).collect(),
    }
}

fn map_current_app_user_profile_response(
    profile: service::dto::app::CurrentAppUserProfileResponse,
) -> crate::dto::app::CurrentAppUserProfileResponse {
    crate::dto::app::CurrentAppUserProfileResponse {
        user_id: profile.user_id,
        display_id: profile.display_id,
        display_name: profile.display_name,
        school_name: profile.school_name,
        team_name: profile.team_name,
        is_school_team_bound: profile.is_school_team_bound,
    }
}

pub async fn register_app_user(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<RegisterAppUserRequest>,
) -> ResponseResult<AppUserResponse> {
    req.validate().map_err(Error::from)?;
    let api = AppApi::new(get_app_ctx());
    let app_user = api
        .register_app_user(service::dto::app::RegisterAppUserRequest {
            user_id: auth_user.user_id,
            display_id: req.display_id,
            display_name: req.display_name,
            school_name: req.school_name,
            team_name: req.team_name,
            remark: req.remark,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_app_user_response(app_user)
        .into_common_response()
        .to_json())
}

pub async fn current_user_permissions(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<CurrentUserPermissionsResponse> {
    let api = AppApi::new(get_app_ctx());
    let resp = api
        .get_current_user_permissions(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(CurrentUserPermissionsResponse {
        user_id: resp.user_id,
        role_codes: resp.role_codes,
        permission_codes: resp.permission_codes,
    }
    .into_common_response()
    .to_json())
}

pub async fn get_current_app_user_profile(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<crate::dto::app::CurrentAppUserProfileResponse> {
    let api = AppApi::new(get_app_ctx());
    let profile = api
        .get_current_app_user_profile(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(map_current_app_user_profile_response(profile)
        .into_common_response()
        .to_json())
}

pub async fn update_current_app_user_profile(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateCurrentAppUserProfileRequest>,
) -> ResponseResult<crate::dto::app::CurrentAppUserProfileResponse> {
    req.validate().map_err(Error::from)?;
    let api = AppApi::new(get_app_ctx());
    let profile = api
        .update_current_app_user_profile(service::dto::app::UpdateCurrentAppUserProfileRequest {
            user_id: auth_user.user_id,
            school_name: req.school_name,
            team_name: req.team_name,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_current_app_user_profile_response(profile)
        .into_common_response()
        .to_json())
}
