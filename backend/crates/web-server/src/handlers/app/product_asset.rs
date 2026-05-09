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

const FILE_UPLOAD_EXPIRES_SECS: u64 = 600;

pub async fn presign_product_asset_upload(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<PresignProductAssetUploadRequest>,
) -> ResponseResult<PresignProductAssetUploadResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductApi::new(get_app_ctx());
    api.get_product(service::dto::product::GetProductRequest {
        user_id: auth_user.user_id.clone(),
        product_id: product_id.clone(),
    })
    .await
    .map_err(from_biz_error)?;

    let storage = get_storage();
    let ext = FileKind::Document
        .normalize_ext(&req.extension)
        .map_err(|err| Error::Custom(err.to_string()))?;
    let content_type = req
        .content_type
        .or_else(|| Some(FileKind::Document.content_type_for_ext(&ext).to_string()));
    let object_key = product_asset_object_key(&auth_user.user_id, &product_id, &ext);
    let signed = storage
        .presign_upload(PresignUploadRequest {
            bucket: BucketKind::Private,
            key: object_key,
            content_type,
            expires_secs: Some(FILE_UPLOAD_EXPIRES_SECS),
        })
        .map_err(|err| Error::Custom(err.to_string()))?;

    Ok(PresignProductAssetUploadResponse {
        method: signed.method,
        upload_url: signed.upload_url,
        bucket: signed.bucket,
        object_key: signed.key,
        content_type: signed.content_type,
        expires_secs: signed.expires_secs,
    }
    .into_common_response()
    .to_json())
}

pub async fn list_product_assets(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Query(req): Query<ListProductAssetsQuery>,
) -> ResponseResult<Vec<ProductAssetResponse>> {
    req.validate().map_err(Error::from)?;
    let api = ProductAssetApi::new(get_app_ctx());
    let assets: Vec<ProductAssetResponse> = api
        .list_assets(service::dto::product_asset::ListProductAssetsRequest {
            user_id: auth_user.user_id,
            product_id,
            status: req.status,
            asset_type: req.asset_type,
            keyword: req.keyword,
        })
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_product_asset_response)
        .collect();

    Ok(assets.into_common_response().to_json())
}

pub async fn register_product_asset(
    Extension(auth_user): Extension<AuthUser>,
    Path(product_id): Path<String>,
    Json(req): Json<RegisterProductAssetRequest>,
) -> ResponseResult<ProductAssetResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductAssetApi::new(get_app_ctx());
    let asset = api
        .register_asset(service::dto::product_asset::RegisterProductAssetRequest {
            user_id: auth_user.user_id,
            product_id,
            bucket: req.bucket,
            object_key: req.object_key,
            asset_type: req.asset_type,
            file_name: req.file_name,
            mime_type: req.mime_type,
            file_size: req.file_size,
            description: req.description,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_asset_response(asset)
        .into_common_response()
        .to_json())
}

pub async fn get_product_asset_access(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, asset_id)): Path<(String, String)>,
) -> ResponseResult<ProductAssetAccessResponse> {
    let api = ProductAssetApi::new(get_app_ctx());
    let asset = api
        .get_asset(service::dto::product_asset::GetProductAssetRequest {
            user_id: auth_user.user_id,
            product_id,
            asset_id,
        })
        .await
        .map_err(from_biz_error)?;

    let storage = get_storage();
    let signed = storage
        .presign_download(PresignDownloadRequest {
            bucket: BucketKind::Private,
            key: asset.object_key.clone(),
            expires_secs: Some(FILE_UPLOAD_EXPIRES_SECS),
        })
        .map_err(|err| Error::Custom(err.to_string()))?;

    Ok(ProductAssetAccessResponse {
        asset: map_product_asset_response(asset),
        method: signed.method,
        download_url: signed.download_url,
        expires_secs: FILE_UPLOAD_EXPIRES_SECS,
    }
    .into_common_response()
    .to_json())
}

pub async fn update_product_asset(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, asset_id)): Path<(String, String)>,
    Json(req): Json<UpdateProductAssetRequest>,
) -> ResponseResult<ProductAssetResponse> {
    req.validate().map_err(Error::from)?;
    let api = ProductAssetApi::new(get_app_ctx());
    let asset = api
        .update_asset(service::dto::product_asset::UpdateProductAssetRequest {
            user_id: auth_user.user_id,
            product_id,
            asset_id,
            asset_type: req.asset_type,
            file_name: req.file_name,
            mime_type: req.mime_type,
            file_size: req.file_size,
            description: req.description,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_product_asset_response(asset)
        .into_common_response()
        .to_json())
}

pub async fn delete_product_asset(
    Extension(auth_user): Extension<AuthUser>,
    Path((product_id, asset_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = ProductAssetApi::new(get_app_ctx());
    api.delete_asset(service::dto::product_asset::DeleteProductAssetRequest {
        user_id: auth_user.user_id,
        product_id,
        asset_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

fn map_product_asset_response(
    item: service::dto::product_asset::ProductAssetResponse,
) -> ProductAssetResponse {
    ProductAssetResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        bucket: item.bucket,
        object_key: item.object_key,
        asset_type: item.asset_type,
        file_name: item.file_name,
        mime_type: item.mime_type,
        file_size: item.file_size,
        description: item.description,
        status: item.status,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
