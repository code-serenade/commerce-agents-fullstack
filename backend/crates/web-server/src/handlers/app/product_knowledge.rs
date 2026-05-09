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

pub async fn list_product_knowledge(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Query(req): Query<ListProductKnowledgeQuery>,
) -> ResponseResult<Vec<ProductKnowledgeResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductKnowledgeApi::new(get_app_ctx());
    let knowledge: Vec<ProductKnowledgeResponse> = api
        .list_knowledge(
            service::dto::product_knowledge::ListProductKnowledgeRequest {
                user_id: auth_user.user_id,
                product_id,
                status: req.status,
                content_type: req.content_type,
                keyword: req.keyword,
            },
        )
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_knowledge_response)
        .collect();

    Ok(knowledge.into_common_response().to_json())
}

pub async fn create_product_knowledge(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateProductKnowledgeRequest>,
) -> ResponseResult<ProductKnowledgeResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductKnowledgeApi::new(get_app_ctx());
    let item = api
        .create_knowledge(
            service::dto::product_knowledge::CreateProductKnowledgeRequest {
                user_id: auth_user.user_id,
                product_id,
                title: req.title,
                content_type: req.content_type,
                content: req.content,
                source_type: req.source_type,
                source_id: req.source_id,
                confidence: req.confidence,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_knowledge_response(item)
        .into_common_response()
        .to_json())
}

pub async fn update_product_knowledge(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, knowledge_id)): Path<(String, String)>,
    Json(req): Json<UpdateProductKnowledgeRequest>,
) -> ResponseResult<ProductKnowledgeResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductKnowledgeApi::new(get_app_ctx());
    let item = api
        .update_knowledge(
            service::dto::product_knowledge::UpdateProductKnowledgeRequest {
                user_id: auth_user.user_id,
                product_id,
                knowledge_id,
                title: req.title,
                content_type: req.content_type,
                content: req.content,
                source_type: req.source_type,
                source_id: req.source_id,
                confidence: req.confidence,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_knowledge_response(item)
        .into_common_response()
        .to_json())
}

pub async fn delete_product_knowledge(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, knowledge_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = ProductKnowledgeApi::new(get_app_ctx());
    api.delete_knowledge(
        service::dto::product_knowledge::DeleteProductKnowledgeRequest {
            user_id: auth_user.user_id,
            product_id,
            knowledge_id,
        },
    )
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

fn map_product_knowledge_response(
    item: service::dto::product_knowledge::ProductKnowledgeResponse,
) -> ProductKnowledgeResponse {
    ProductKnowledgeResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        title: item.title,
        content_type: item.content_type,
        content: item.content,
        source_type: item.source_type,
        source_id: item.source_id,
        confidence: item.confidence,
        status: item.status,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
