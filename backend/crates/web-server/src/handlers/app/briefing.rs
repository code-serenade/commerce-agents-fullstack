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

pub async fn daily_briefing(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<DailyBriefingQuery>,
) -> ResponseResult<DailyBriefingSummaryResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .daily_briefing(service::dto::briefing_ops::GetDailyBriefingRequest {
            user_id: auth_user.user_id,
            date: req.date,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_daily_briefing_records(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListDailyBriefingRecordsQuery>,
) -> ResponseResult<DailyBriefingListResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .list_daily_briefing_records(
            service::dto::briefing_ops::ListDailyBriefingRecordsRequest {
                user_id: auth_user.user_id,
                archived: req.archived,
                status: req.status,
                date_from: req.date_from,
                date_to: req.date_to,
                page: req.page,
                page_size: req.page_size,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn upsert_daily_briefing_record(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpsertDailyBriefingRecordRequest>,
) -> ResponseResult<DailyBriefingResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .upsert_daily_briefing_record(
            service::dto::briefing_ops::UpsertDailyBriefingRecordRequest {
                user_id: auth_user.user_id,
                briefing_date: req.briefing_date,
                title: req.title,
                summary: req.summary,
                sections: req.sections,
                product_summary: req.product_summary,
                alert_summary: req.alert_summary,
                task_summary: req.task_summary,
                source: req.source,
                status: req.status,
                archived: req.archived,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_daily_briefing_record(
    Extension(auth_user): Extension<AuthUser>,
    Path(briefing_id): Path<String>,
) -> ResponseResult<DailyBriefingResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .get_daily_briefing_record(service::dto::briefing_ops::GetDailyBriefingRecordRequest {
            user_id: auth_user.user_id,
            briefing_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn archive_daily_briefing_record(
    Extension(auth_user): Extension<AuthUser>,
    Path(briefing_id): Path<String>,
) -> ResponseResult<DailyBriefingResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .archive_daily_briefing_record(
            service::dto::briefing_ops::ArchiveDailyBriefingRecordRequest {
                user_id: auth_user.user_id,
                briefing_id,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_background_events(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListBackgroundEventsQuery>,
) -> ResponseResult<BackgroundEventListResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .list_background_events(service::dto::briefing_ops::ListBackgroundEventsRequest {
            user_id: auth_user.user_id,
            conversation_id: req.conversation_id,
            event_type: req.event_type,
            consumed: req.consumed,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_background_event(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateBackgroundEventRequest>,
) -> ResponseResult<BackgroundEventResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .create_background_event(service::dto::briefing_ops::CreateBackgroundEventRequest {
            user_id: auth_user.user_id,
            conversation_id: req.conversation_id,
            event_type: req.event_type,
            title: req.title,
            data: req.data,
            consumed: req.consumed,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn consume_background_event(
    Extension(auth_user): Extension<AuthUser>,
    Path(event_id): Path<String>,
) -> ResponseResult<BackgroundEventResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .consume_background_event(service::dto::briefing_ops::ConsumeBackgroundEventRequest {
            user_id: auth_user.user_id,
            event_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn consume_background_events(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<ConsumeBackgroundEventsRequest>,
) -> ResponseResult<BatchUpdateResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .consume_background_events(service::dto::briefing_ops::ConsumeBackgroundEventsRequest {
            user_id: auth_user.user_id,
            conversation_id: req.conversation_id,
            event_type: req.event_type,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_alerts(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListAlertsQuery>,
) -> ResponseResult<AlertListResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .list_alerts(service::dto::briefing_ops::ListAlertsRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            alert_type: req.alert_type,
            severity: req.severity,
            status: req.status,
            include_snoozed: req.include_snoozed,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_alert(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAlertRequest>,
) -> ResponseResult<AlertResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .create_alert(service::dto::briefing_ops::CreateAlertRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            alert_type: req.alert_type,
            severity: req.severity,
            message: req.message,
            role: req.role,
            dedup_key: req.dedup_key,
            snoozed_until: req.snoozed_until,
            payload: req.payload,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn alert_summary(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<AlertSummaryResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .alert_summary(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn dismiss_all_alerts(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<DismissAllAlertsRequest>,
) -> ResponseResult<BatchUpdateResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .dismiss_all_alerts(service::dto::briefing_ops::DismissAllAlertsRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            alert_type: req.alert_type,
            severity: req.severity,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_alert(
    Extension(auth_user): Extension<AuthUser>,
    Path(alert_id): Path<String>,
) -> ResponseResult<AlertResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .get_alert(service::dto::briefing_ops::GetAlertRequest {
            user_id: auth_user.user_id,
            alert_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn snooze_alert(
    Extension(auth_user): Extension<AuthUser>,
    Path(alert_id): Path<String>,
    Json(req): Json<SnoozeAlertRequest>,
) -> ResponseResult<AlertResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .snooze_alert(service::dto::briefing_ops::SnoozeAlertRequest {
            user_id: auth_user.user_id,
            alert_id,
            snoozed_until: req.snoozed_until,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn dismiss_alert(
    Extension(auth_user): Extension<AuthUser>,
    Path(alert_id): Path<String>,
    Json(req): Json<DismissAlertRequest>,
) -> ResponseResult<AlertResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .dismiss_alert(service::dto::briefing_ops::DismissAlertRequest {
            user_id: auth_user.user_id,
            alert_id,
            reason: req.reason,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn act_alert(
    Extension(auth_user): Extension<AuthUser>,
    Path(alert_id): Path<String>,
    Json(req): Json<ActAlertRequest>,
) -> ResponseResult<AlertResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .act_alert(service::dto::briefing_ops::ActAlertRequest {
            user_id: auth_user.user_id,
            alert_id,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}
