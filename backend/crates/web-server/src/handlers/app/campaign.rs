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
        CampaignDraftResponse, CampaignDraftSourceResponse, CampaignFormDraftResponse,
        CampaignListResponse, CampaignResponse, CampaignTemplateImportResponse,
        CampaignTemplateResponse, CancelLongTaskRequest, ChannelCatalogResponse,
        ChannelConnectionListResponse, ChannelConnectionResponse, CompetitorChangeResponse,
        ConsumeBackgroundEventsRequest, ConversationDetailResponse, ConversationFileAccessResponse,
        ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
        CreateAlertRequest, CreateAuditLogRequest, CreateBackgroundEventRequest,
        CreateCampaignDraftRequest, CreateCampaignFormDraftRequest, CreateCampaignRequest,
        CreateCompetitorChangeRequest, CreateConversationRequest, CreateExecutionQueueRequest,
        CreateLongTaskRequest, CreateProductCompetitorRequest, CreateProductKnowledgeRequest,
        CreateProductMaterialRequest, CreateProductRequest, CreateToolApprovalRequest,
        CreateWorkspaceRequest, CreateWorkspaceRunRequest, CreateWorkspaceRunTaskRequest,
        CreateWorkspaceTaskRequest, CurrentUserPermissionsResponse, DailyBriefingListResponse,
        DailyBriefingQuery, DailyBriefingResponse, DailyBriefingSummaryResponse,
        DecideExecutionQueueRequest, DecideToolApprovalRequest, DismissAlertRequest,
        DismissAllAlertsRequest, ExecutionQueueListResponse, ExecutionQueueResponse,
        ImportCampaignTemplateRequest, ListAlertsQuery, ListAuditLogsQuery,
        ListBackgroundEventsQuery, ListCampaignsQuery, ListChannelConnectionsQuery,
        ListCompetitorChangesQuery, ListDailyBriefingRecordsQuery, ListExecutionQueueQuery,
        ListLongTasksQuery, ListPackageCatalogQuery, ListPackageHistoryQuery,
        ListPlatformConnectionsQuery, ListProductAssetsQuery, ListProductCompetitorsQuery,
        ListProductKnowledgeQuery, ListProductMaterialsQuery, ListProductsQuery,
        ListStoreMetricsQuery, ListToolApprovalsQuery, ListWorkspaceRunTasksQuery,
        ListWorkspaceRunsQuery, ListWorkspaceTasksQuery, ListWorkspacesQuery, LongTaskListResponse,
        LongTaskResponse, PackageCatalogResponse, PackageHistoryListResponse,
        PackageLockDriftResponse, PackageLockListResponse, PackageResponse,
        PackageStatusChangeRequest, PackageValidationResponse, PlatformCatalogResponse,
        PlatformConnectionListResponse, PlatformConnectionResponse, PlatformSummaryResponse,
        PlatformSyncStatusListResponse, PresignConversationFileUploadRequest,
        PresignConversationFileUploadResponse, PresignProductAssetUploadRequest,
        PresignProductAssetUploadResponse, ProductAssetAccessResponse, ProductAssetResponse,
        ProductCompetitorResponse, ProductDashboardResponse, ProductDetailResponse,
        ProductKnowledgeResponse, ProductMaterialResponse, ProductResponse, RegisterAppUserRequest,
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
    handlers::app::form_draft::{
        FormDraftGenerationMode, campaign_form_draft_input, form_draft_agent_failure_error,
        map_agent_campaign_form_draft, parse_form_draft_generation_mode,
    },
    statics::{
        agent_manager::run_form_draft, db_manager::get_app_ctx, storage_manager::get_storage,
    },
};

pub async fn list_campaigns(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListCampaignsQuery>,
) -> ResponseResult<CampaignListResponse> {
    req.validate().map_err(Error::from)?;
    let api = CampaignApi::new(get_app_ctx());
    let campaigns = api
        .list_campaigns(service::dto::campaign::ListCampaignsRequest {
            user_id: auth_user.user_id,
            status: req.status,
            product_id: req.product_id,
            campaign_type: req.campaign_type,
            keyword: req.keyword,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_campaign_list_response(campaigns)
        .into_common_response()
        .to_json())
}

pub async fn create_campaign(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCampaignRequest>,
) -> ResponseResult<CampaignResponse> {
    req.validate().map_err(Error::from)?;
    let api = CampaignApi::new(get_app_ctx());
    let campaign = api
        .create_campaign(service::dto::campaign::CreateCampaignRequest {
            user_id: auth_user.user_id,
            product_id: req.product_id,
            name: req.name,
            campaign_type: req.campaign_type,
            description: req.description,
            budget: req.budget,
            start_date: req.start_date,
            end_date: req.end_date,
            status: req.status,
            metadata: req.metadata,
            output: req.output,
            action_metadata: req.action_metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_campaign_response(campaign)
        .into_common_response()
        .to_json())
}

pub async fn list_campaign_templates(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<CampaignTemplateResponse>> {
    let api = CampaignApi::new(get_app_ctx());
    let templates = api
        .list_templates(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(templates.into_common_response().to_json())
}

pub async fn import_campaign_template(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<ImportCampaignTemplateRequest>,
) -> ResponseResult<CampaignTemplateImportResponse> {
    req.validate().map_err(Error::from)?;
    let api = CampaignApi::new(get_app_ctx());
    let imported = api
        .import_template(service::dto::campaign::ImportCampaignTemplateRequest {
            user_id: auth_user.user_id,
            template_id: req.template_id,
            product_id: req.product_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(imported.into_common_response().to_json())
}

pub async fn create_campaign_form_draft(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCampaignFormDraftRequest>,
) -> ResponseResult<CampaignFormDraftResponse> {
    req.validate().map_err(Error::from)?;
    let generation_mode =
        parse_form_draft_generation_mode(req.generation_mode.as_deref()).map_err(Error::Custom)?;
    if generation_mode == FormDraftGenerationMode::AgentLlm {
        let result = run_form_draft(campaign_form_draft_input(&req))
            .await
            .map_err(form_draft_agent_failure_error)?;
        return Ok(map_agent_campaign_form_draft(result)
            .into_common_response()
            .to_json());
    }

    let api = CampaignApi::new(get_app_ctx());
    let draft = api
        .create_form_draft(service::dto::campaign::CreateCampaignFormDraftRequest {
            user_id: auth_user.user_id,
            product_id: req.product_id,
            name: req.name,
            campaign_type: req.campaign_type,
            description: req.description,
            budget: req.budget,
            start_date: req.start_date,
            end_date: req.end_date,
            goal: req.goal,
            target_audience: req.target_audience,
            channels: req.channels,
            prompt: req.prompt,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(draft.into_common_response().to_json())
}

pub async fn create_campaign_draft(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateCampaignDraftRequest>,
) -> ResponseResult<CampaignDraftResponse> {
    req.validate().map_err(Error::from)?;
    let api = CampaignApi::new(get_app_ctx());
    let draft = api
        .create_campaign_draft(service::dto::campaign::CreateCampaignDraftRequest {
            user_id: auth_user.user_id,
            product_id: req.product_id,
            workspace_id: req.workspace_id,
            source_artifact_id: req.source_artifact_id,
            name: req.name,
            campaign_type: req.campaign_type,
            description: req.description,
            budget: req.budget,
            start_date: req.start_date,
            end_date: req.end_date,
            goal: req.goal,
            target_audience: req.target_audience,
            channels: req.channels,
            strategy: req.strategy,
            source_type: req.source_type,
            source_summary: req.source_summary,
            draft: req.draft,
            metadata: req.metadata,
            action_metadata: req.action_metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_campaign_draft_response(draft)
        .into_common_response()
        .to_json())
}

pub async fn get_campaign(
    Extension(auth_user): Extension<AuthUser>,
    Path(campaign_id): Path<String>,
) -> ResponseResult<CampaignResponse> {
    let api = CampaignApi::new(get_app_ctx());
    let campaign = api
        .get_campaign(service::dto::campaign::GetCampaignRequest {
            user_id: auth_user.user_id,
            campaign_id,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_campaign_response(campaign)
        .into_common_response()
        .to_json())
}

pub async fn update_campaign(
    Extension(auth_user): Extension<AuthUser>,
    Path(campaign_id): Path<String>,
    Json(req): Json<UpdateCampaignRequest>,
) -> ResponseResult<CampaignResponse> {
    req.validate().map_err(Error::from)?;
    let api = CampaignApi::new(get_app_ctx());
    let campaign = api
        .update_campaign(service::dto::campaign::UpdateCampaignRequest {
            user_id: auth_user.user_id,
            campaign_id,
            product_id: req.product_id,
            name: req.name,
            campaign_type: req.campaign_type,
            description: req.description,
            budget: req.budget,
            start_date: req.start_date,
            end_date: req.end_date,
            status: req.status,
            metadata: req.metadata,
            output: req.output,
            action_metadata: req.action_metadata,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_campaign_response(campaign)
        .into_common_response()
        .to_json())
}

pub async fn delete_campaign(
    Extension(auth_user): Extension<AuthUser>,
    Path(campaign_id): Path<String>,
) -> ResponseResult<()> {
    let api = CampaignApi::new(get_app_ctx());
    api.delete_campaign(service::dto::campaign::DeleteCampaignRequest {
        user_id: auth_user.user_id,
        campaign_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

fn map_campaign_list_response(
    item: service::dto::campaign::CampaignListResponse,
) -> CampaignListResponse {
    CampaignListResponse {
        campaigns: item
            .campaigns
            .into_iter()
            .map(map_campaign_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_campaign_response(item: service::dto::campaign::CampaignResponse) -> CampaignResponse {
    CampaignResponse {
        id: item.id,
        user_id: item.user_id,
        product_id: item.product_id,
        name: item.name,
        campaign_type: item.campaign_type,
        description: item.description,
        budget: item.budget,
        start_date: item.start_date,
        end_date: item.end_date,
        status: item.status,
        metadata: item.metadata,
        output: item.output,
        action_metadata: item.action_metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_campaign_draft_response(
    item: service::dto::campaign::CampaignDraftResponse,
) -> CampaignDraftResponse {
    CampaignDraftResponse {
        campaign: map_campaign_response(item.campaign),
        source: CampaignDraftSourceResponse {
            source_type: item.source.source_type,
            product_id: item.source.product_id,
            workspace_id: item.source.workspace_id,
            source_artifact_id: item.source.source_artifact_id,
            source_summary: item.source.source_summary,
            ref_keys: item.source.ref_keys,
        },
        warnings: item.warnings,
        agent_context: item.agent_context,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_form_draft_rejects_unknown_generation_mode() {
        let err = parse_form_draft_generation_mode(Some("llm")).unwrap_err();

        assert!(err.contains("unsupported form draft generationMode"));
    }
}
