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

pub async fn list_products(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListProductsQuery>,
) -> ResponseResult<Vec<ProductResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductApi::new(get_app_ctx());
    let products: Vec<ProductResponse> = api
        .list_products(service::dto::product::ListProductsRequest {
            user_id: auth_user.user_id,
            status: req.status,
            keyword: req.keyword,
        })
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_response)
        .collect();

    Ok(products.into_common_response().to_json())
}

pub async fn create_product(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateProductRequest>,
) -> ResponseResult<ProductResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductApi::new(get_app_ctx());
    let product = api
        .create_product(service::dto::product::CreateProductRequest {
            user_id: auth_user.user_id,
            name: req.name,
            platform: req.platform,
            external_id: req.external_id,
            category: req.category,
            brand: req.brand,
            description: req.description,
            price: req.price,
            currency: req.currency,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_response(product)
        .into_common_response()
        .to_json())
}

pub async fn get_product(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
) -> ResponseResult<ProductDetailResponse> {
    let api = ProductApi::new(get_app_ctx());
    let detail = api
        .get_product(service::dto::product::GetProductRequest {
            user_id: auth_user.user_id,
            product_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(ProductDetailResponse {
        product: map_product_response(detail.product),
    }
    .into_common_response()
    .to_json())
}

pub async fn update_product(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> ResponseResult<ProductResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductApi::new(get_app_ctx());
    let product = api
        .update_product(service::dto::product::UpdateProductRequest {
            user_id: auth_user.user_id,
            product_id,
            name: req.name,
            platform: req.platform,
            external_id: req.external_id,
            category: req.category,
            brand: req.brand,
            description: req.description,
            price: req.price,
            currency: req.currency,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_response(product)
        .into_common_response()
        .to_json())
}

pub async fn delete_product(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
) -> ResponseResult<()> {
    let api = ProductApi::new(get_app_ctx());
    api.delete_product(service::dto::product::DeleteProductRequest {
        user_id: auth_user.user_id,
        product_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn product_dashboard(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<ProductDashboardResponse> {
    let api = ProductApi::new(get_app_ctx());
    let dashboard = api
        .dashboard(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(ProductDashboardResponse {
        total: dashboard.total,
        active: dashboard.active,
        archived: dashboard.archived,
    }
    .into_common_response()
    .to_json())
}

fn map_product_response(item: service::dto::product::ProductResponse) -> ProductResponse {
    ProductResponse {
        id: item.id,
        user_id: item.user_id,
        name: item.name,
        platform: item.platform,
        external_id: item.external_id,
        category: item.category,
        brand: item.brand,
        status: item.status,
        description: item.description,
        price: item.price,
        currency: item.currency,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
