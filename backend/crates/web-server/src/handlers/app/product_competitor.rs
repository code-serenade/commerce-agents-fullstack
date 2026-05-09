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

pub async fn list_product_competitors(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Query(req): Query<ListProductCompetitorsQuery>,
) -> ResponseResult<Vec<ProductCompetitorResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductCompetitorApi::new(get_app_ctx());
    let competitors: Vec<ProductCompetitorResponse> = api
        .list_competitors(
            service::dto::product_competitor::ListProductCompetitorsRequest {
                user_id: auth_user.user_id,
                product_id,
                status: req.status,
                platform: req.platform,
                keyword: req.keyword,
            },
        )
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_competitor_response)
        .collect();

    Ok(competitors.into_common_response().to_json())
}

pub async fn create_product_competitor(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateProductCompetitorRequest>,
) -> ResponseResult<ProductCompetitorResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductCompetitorApi::new(get_app_ctx());
    let competitor = api
        .create_competitor(
            service::dto::product_competitor::CreateProductCompetitorRequest {
                user_id: auth_user.user_id,
                product_id,
                name: req.name,
                platform: req.platform,
                url: req.url,
                price: req.price,
                rating: req.rating,
                monthly_sales: req.monthly_sales,
                notes: req.notes,
                search_results: req.search_results,
                metadata: req.metadata,
                last_searched_at: req.last_searched_at,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_competitor_response(competitor)
        .into_common_response()
        .to_json())
}

pub async fn get_product_competitor(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, competitor_id)): Path<(String, String)>,
) -> ResponseResult<ProductCompetitorResponse> {
    let api = ProductCompetitorApi::new(get_app_ctx());
    let competitor = api
        .get_competitor(
            service::dto::product_competitor::GetProductCompetitorRequest {
                user_id: auth_user.user_id,
                product_id,
                competitor_id,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_competitor_response(competitor)
        .into_common_response()
        .to_json())
}

pub async fn update_product_competitor(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, competitor_id)): Path<(String, String)>,
    Json(req): Json<UpdateProductCompetitorRequest>,
) -> ResponseResult<ProductCompetitorResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductCompetitorApi::new(get_app_ctx());
    let competitor = api
        .update_competitor(
            service::dto::product_competitor::UpdateProductCompetitorRequest {
                user_id: auth_user.user_id,
                product_id,
                competitor_id,
                name: req.name,
                platform: req.platform,
                url: req.url,
                price: req.price,
                rating: req.rating,
                monthly_sales: req.monthly_sales,
                notes: req.notes,
                search_results: req.search_results,
                metadata: req.metadata,
                last_searched_at: req.last_searched_at,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_competitor_response(competitor)
        .into_common_response()
        .to_json())
}

pub async fn delete_product_competitor(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, competitor_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = ProductCompetitorApi::new(get_app_ctx());
    api.delete_competitor(
        service::dto::product_competitor::DeleteProductCompetitorRequest {
            user_id: auth_user.user_id,
            product_id,
            competitor_id,
        },
    )
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn list_competitor_changes(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Query(req): Query<ListCompetitorChangesQuery>,
) -> ResponseResult<Vec<CompetitorChangeResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductCompetitorApi::new(get_app_ctx());
    let changes: Vec<CompetitorChangeResponse> = api
        .list_changes(
            service::dto::product_competitor::ListCompetitorChangesRequest {
                user_id: auth_user.user_id,
                product_id,
                competitor_id: req.competitor_id,
                change_type: req.change_type,
                limit: req.limit,
            },
        )
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_competitor_change_response)
        .collect();

    Ok(changes.into_common_response().to_json())
}

pub async fn create_competitor_change(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, competitor_id)): Path<(String, String)>,
    Json(req): Json<CreateCompetitorChangeRequest>,
) -> ResponseResult<CompetitorChangeResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductCompetitorApi::new(get_app_ctx());
    let change = api
        .create_change(
            service::dto::product_competitor::CreateCompetitorChangeRequest {
                user_id: auth_user.user_id,
                product_id,
                competitor_id,
                change_type: req.change_type,
                old_value: req.old_value,
                new_value: req.new_value,
                summary: req.summary,
                source_type: req.source_type,
                metadata: req.metadata,
                detected_at: req.detected_at,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_competitor_change_response(change)
        .into_common_response()
        .to_json())
}

fn map_product_competitor_response(
    item: service::dto::product_competitor::ProductCompetitorResponse,
) -> ProductCompetitorResponse {
    ProductCompetitorResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        name: item.name,
        platform: item.platform,
        url: item.url,
        price: item.price,
        rating: item.rating,
        monthly_sales: item.monthly_sales,
        notes: item.notes,
        search_results: item.search_results,
        status: item.status,
        metadata: item.metadata,
        last_searched_at: item.last_searched_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_competitor_change_response(
    item: service::dto::product_competitor::CompetitorChangeResponse,
) -> CompetitorChangeResponse {
    CompetitorChangeResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        competitor_id: item.competitor_id,
        competitor_name: item.competitor_name,
        competitor_platform: item.competitor_platform,
        change_type: item.change_type,
        old_value: item.old_value,
        new_value: item.new_value,
        summary: item.summary,
        source_type: item.source_type,
        metadata: item.metadata,
        detected_at: item.detected_at,
        created_at: item.created_at,
    }
}
