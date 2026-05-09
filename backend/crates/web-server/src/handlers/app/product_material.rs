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

pub async fn list_product_materials(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Query(req): Query<ListProductMaterialsQuery>,
) -> ResponseResult<Vec<ProductMaterialResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductMaterialApi::new(get_app_ctx());
    let materials: Vec<ProductMaterialResponse> = api
        .list_materials(
            service::dto::product_material::ListProductMaterialsRequest {
                user_id: auth_user.user_id,
                product_id,
                status: req.status,
                material_type: req.material_type,
                keyword: req.keyword,
            },
        )
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_material_response)
        .collect();

    Ok(materials.into_common_response().to_json())
}

pub async fn create_product_material(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateProductMaterialRequest>,
) -> ResponseResult<ProductMaterialResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductMaterialApi::new(get_app_ctx());
    let material = api
        .create_material(
            service::dto::product_material::CreateProductMaterialRequest {
                user_id: auth_user.user_id,
                product_id,
                material_type: req.material_type,
                title: req.title,
                content: req.content,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_material_response(material)
        .into_common_response()
        .to_json())
}

pub async fn get_product_material(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, material_id)): Path<(String, String)>,
) -> ResponseResult<ProductMaterialResponse> {
    let api = ProductMaterialApi::new(get_app_ctx());
    let material = api
        .get_material(service::dto::product_material::GetProductMaterialRequest {
            user_id: auth_user.user_id,
            product_id,
            material_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_material_response(material)
        .into_common_response()
        .to_json())
}

pub async fn update_product_material(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, material_id)): Path<(String, String)>,
    Json(req): Json<UpdateProductMaterialRequest>,
) -> ResponseResult<ProductMaterialResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductMaterialApi::new(get_app_ctx());
    let material = api
        .update_material(
            service::dto::product_material::UpdateProductMaterialRequest {
                user_id: auth_user.user_id,
                product_id,
                material_id,
                title: req.title,
                content: req.content,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_material_response(material)
        .into_common_response()
        .to_json())
}

pub async fn delete_product_material(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, material_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = ProductMaterialApi::new(get_app_ctx());
    api.delete_material(
        service::dto::product_material::DeleteProductMaterialRequest {
            user_id: auth_user.user_id,
            product_id,
            material_id,
        },
    )
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn list_product_material_versions(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, material_id)): Path<(String, String)>,
) -> ResponseResult<Vec<ProductMaterialResponse>> {
    let api = ProductMaterialApi::new(get_app_ctx());
    let materials: Vec<ProductMaterialResponse> = api
        .material_versions(service::dto::product_material::GetProductMaterialRequest {
            user_id: auth_user.user_id,
            product_id,
            material_id,
        })
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_material_response)
        .collect();

    Ok(materials.into_common_response().to_json())
}

fn map_product_material_response(
    item: service::dto::product_material::ProductMaterialResponse,
) -> ProductMaterialResponse {
    ProductMaterialResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        material_type: item.material_type,
        title: item.title,
        version: item.version,
        content: item.content,
        status: item.status,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
