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
        ActAlertRequest, AddConversationMessageRequest, AgentLearningRecordListResponse,
        AgentQualityCheckListResponse, AgentQualityEvaluationResponse, AgentRunFeedbackResponse,
        AgentSkillRunListResponse, AgentTrustScoreListResponse, AlertListResponse, AlertResponse,
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
        CreateToolApprovalRequest, CreateWorkspaceArtifactRequest, CreateWorkspaceRequest,
        CreateWorkspaceRunRequest, CreateWorkspaceRunTaskRequest, CreateWorkspaceTaskRequest,
        CurrentUserPermissionsResponse, DailyBriefingListResponse, DailyBriefingQuery,
        DailyBriefingResponse, DailyBriefingSummaryResponse, DecideExecutionQueueRequest,
        DecideToolApprovalRequest, DismissAlertRequest, DismissAllAlertsRequest,
        EvaluateAgentRunQualityRequest, ExecutionQueueListResponse, ExecutionQueueResponse,
        ExportWorkspaceArtifactQuery, ListAgentLearningRecordsQuery, ListAgentQualityChecksQuery,
        ListAgentSkillRunsQuery, ListAgentTrustScoresQuery, ListAlertsQuery, ListAuditLogsQuery,
        ListBackgroundEventsQuery, ListCampaignsQuery, ListChannelConnectionsQuery,
        ListCompetitorChangesQuery, ListDailyBriefingRecordsQuery, ListExecutionQueueQuery,
        ListLongTasksQuery, ListPackageCatalogQuery, ListPackageHistoryQuery,
        ListPlatformConnectionsQuery, ListProductAssetsQuery, ListProductCompetitorsQuery,
        ListProductKnowledgeQuery, ListProductMaterialsQuery, ListProductsQuery,
        ListStoreMetricsQuery, ListToolApprovalsQuery, ListWorkspaceArtifactsQuery,
        ListWorkspaceRunTasksQuery, ListWorkspaceRunsQuery, ListWorkspaceTasksQuery,
        ListWorkspacesQuery, LongTaskListResponse, LongTaskResponse, PackageCatalogResponse,
        PackageHistoryListResponse, PackageLockDriftResponse, PackageLockListResponse,
        PackageResponse, PackageStatusChangeRequest, PackageValidationResponse,
        PlatformCatalogResponse, PlatformConnectionListResponse, PlatformConnectionResponse,
        PlatformSummaryResponse, PlatformSyncStatusListResponse,
        PresignConversationFileUploadRequest, PresignConversationFileUploadResponse,
        PresignProductAssetUploadRequest, PresignProductAssetUploadResponse,
        ProductAssetAccessResponse, ProductAssetResponse, ProductCompetitorResponse,
        ProductDashboardResponse, ProductDetailResponse, ProductKnowledgeResponse,
        ProductMaterialResponse, ProductResponse, RegisterAppUserRequest,
        RegisterConversationFileRequest, RegisterProductAssetRequest, SaveChannelConnectionRequest,
        SavePackageLocksRequest, SavePlatformConnectionRequest, SaveStoreMetricRequest,
        SnoozeAlertRequest, StoreMetricListResponse, StoreMetricResponse,
        SubmitAgentRunFeedbackRequest, ToolApprovalListResponse, ToolApprovalResponse,
        UpdateCampaignRequest, UpdateChannelConnectionRequest, UpdateConversationRequest,
        UpdateLongTaskRequest, UpdatePackageRequest, UpdatePlatformConnectionRequest,
        UpdateProductAssetRequest, UpdateProductCompetitorRequest, UpdateProductKnowledgeRequest,
        UpdateProductMaterialRequest, UpdateProductRequest, UpdateWorkspaceArtifactRequest,
        UpdateWorkspaceRequest, UpdateWorkspaceRunRequest, UpdateWorkspaceRunTaskRequest,
        UpdateWorkspaceTaskRequest, UpsertDailyBriefingRecordRequest, UpsertPackageRequest,
        WorkspaceArtifactExportResponse, WorkspaceArtifactListResponse, WorkspaceArtifactResponse,
        WorkspaceContextPackQuery, WorkspaceContextPackResponse, WorkspaceContextSummaryQuery,
        WorkspaceContextSummaryResponse, WorkspaceListResponse, WorkspaceResponse,
        WorkspaceRunListResponse, WorkspaceRunResponse, WorkspaceRunTaskResponse,
        WorkspaceTaskListResponse, WorkspaceTaskResponse,
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

pub async fn list_agent_skill_runs(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Query(req): Query<ListAgentSkillRunsQuery>,
) -> ResponseResult<AgentSkillRunListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_agent_skill_runs(service::dto::workspace::ListAgentSkillRunsRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            status: req.status,
            skill_id: req.skill_id,
            role: req.role,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
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

pub async fn list_workspace_artifacts(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<ListWorkspaceArtifactsQuery>,
) -> ResponseResult<WorkspaceArtifactListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_artifacts(service::dto::workspace::ListWorkspaceArtifactsRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id: req.run_id,
            workspace_task_id: req.workspace_task_id,
            run_task_id: req.run_task_id,
            artifact_type: req.artifact_type,
            status: req.status,
            keyword: req.keyword,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_artifact_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_workspace_artifact(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Json(req): Json<CreateWorkspaceArtifactRequest>,
) -> ResponseResult<WorkspaceArtifactResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_artifact(service::dto::workspace::CreateWorkspaceArtifactRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id: req.run_id,
            workspace_task_id: req.workspace_task_id,
            run_task_id: req.run_task_id,
            artifact_type: req.artifact_type,
            title: req.title,
            content: req.content,
            preview: req.preview,
            mime_type: req.mime_type,
            format: req.format,
            storage_kind: req.storage_kind,
            bucket: req.bucket,
            object_key: req.object_key,
            file_name: req.file_name,
            file_size: req.file_size,
            sha256: req.sha256,
            source: req.source,
            validation_status: req.validation_status,
            validation_report: req.validation_report,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_artifact_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_workspace_artifact(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, artifact_id)): Path<(String, String)>,
) -> ResponseResult<WorkspaceArtifactResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_artifact(service::dto::workspace::GetWorkspaceArtifactRequest {
            user_id: auth_user.user_id,
            workspace_id,
            artifact_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_artifact_response(response)
        .into_common_response()
        .to_json())
}

pub async fn update_workspace_artifact(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, artifact_id)): Path<(String, String)>,
    Json(req): Json<UpdateWorkspaceArtifactRequest>,
) -> ResponseResult<WorkspaceArtifactResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .update_artifact(service::dto::workspace::UpdateWorkspaceArtifactRequest {
            user_id: auth_user.user_id,
            workspace_id,
            artifact_id,
            title: req.title,
            content: req.content,
            preview: req.preview,
            mime_type: req.mime_type,
            format: req.format,
            storage_kind: req.storage_kind,
            bucket: req.bucket,
            object_key: req.object_key,
            file_name: req.file_name,
            file_size: req.file_size,
            sha256: req.sha256,
            status: req.status,
            validation_status: req.validation_status,
            validation_report: req.validation_report,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_workspace_artifact_response(response)
        .into_common_response()
        .to_json())
}

pub async fn delete_workspace_artifact(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, artifact_id)): Path<(String, String)>,
) -> ResponseResult<()> {
    let api = WorkspaceApi::new(get_app_ctx());
    api.delete_artifact(service::dto::workspace::DeleteWorkspaceArtifactRequest {
        user_id: auth_user.user_id,
        workspace_id,
        artifact_id,
    })
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn export_workspace_artifact(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, artifact_id)): Path<(String, String)>,
    Query(req): Query<ExportWorkspaceArtifactQuery>,
) -> ResponseResult<WorkspaceArtifactExportResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .export_artifact(service::dto::workspace::ExportWorkspaceArtifactRequest {
            user_id: auth_user.user_id,
            workspace_id,
            artifact_id,
            format: req.format,
            include_metadata: req.include_metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_workspace_context_summary(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<WorkspaceContextSummaryQuery>,
) -> ResponseResult<WorkspaceContextSummaryResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .context_summary(service::dto::workspace::GetWorkspaceContextSummaryRequest {
            user_id: auth_user.user_id,
            workspace_id,
            conversation_id: req.conversation_id,
            product_id: req.product_id,
            include_workspace_product_context: true,
            run_id: req.run_id,
            context_refs: parse_context_refs_query(req.context_refs)?,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_workspace_context_pack(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<WorkspaceContextPackQuery>,
) -> ResponseResult<WorkspaceContextPackResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .context_pack(service::dto::workspace::GetWorkspaceContextPackRequest {
            user_id: auth_user.user_id,
            workspace_id,
            conversation_id: req.conversation_id,
            product_id: req.product_id,
            include_workspace_product_context: true,
            run_id: req.run_id,
            context_refs: parse_context_refs_query(req.context_refs)?,
            max_items: req.max_items,
            max_chars: req.max_chars,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

fn parse_context_refs_query(value: Option<String>) -> Result<Vec<String>, Error> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let mut refs = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if !refs.iter().any(|existing| existing == item) {
            refs.push(item.to_string());
        }
    }
    Ok(refs)
}

pub async fn list_agent_quality_checks(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Query(req): Query<ListAgentQualityChecksQuery>,
) -> ResponseResult<AgentQualityCheckListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_agent_quality_checks(service::dto::workspace::ListAgentQualityChecksRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn evaluate_agent_run_quality(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Json(req): Json<EvaluateAgentRunQualityRequest>,
) -> ResponseResult<AgentQualityEvaluationResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .evaluate_agent_run(service::dto::workspace::EvaluateAgentWorkspaceRunRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            artifact_id: req.artifact_id,
            output: req.output,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn submit_agent_run_feedback(
    Extension(auth_user): Extension<AuthUser>,
    Path((workspace_id, run_id)): Path<(String, String)>,
    Json(req): Json<SubmitAgentRunFeedbackRequest>,
) -> ResponseResult<AgentRunFeedbackResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .submit_agent_run_feedback(service::dto::workspace::SubmitAgentRunFeedbackRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id,
            rating: req.rating,
            comment: req.comment,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_agent_learning_records(
    Extension(auth_user): Extension<AuthUser>,
    Path(workspace_id): Path<String>,
    Query(req): Query<ListAgentLearningRecordsQuery>,
) -> ResponseResult<AgentLearningRecordListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_agent_learning_records(service::dto::workspace::ListAgentLearningRecordsRequest {
            user_id: auth_user.user_id,
            workspace_id,
            run_id: req.run_id,
            role: req.role,
            category: req.category,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_agent_trust_scores(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListAgentTrustScoresQuery>,
) -> ResponseResult<AgentTrustScoreListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_agent_trust_scores(service::dto::workspace::ListAgentTrustScoresRequest {
            user_id: auth_user.user_id,
            role: req.role,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
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

fn map_workspace_artifact_list_response(
    item: service::dto::workspace::WorkspaceArtifactListResponse,
) -> WorkspaceArtifactListResponse {
    WorkspaceArtifactListResponse {
        artifacts: item
            .artifacts
            .into_iter()
            .map(map_workspace_artifact_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_workspace_artifact_response(
    item: service::dto::workspace::WorkspaceArtifactResponse,
) -> WorkspaceArtifactResponse {
    WorkspaceArtifactResponse {
        id: item.id,
        workspace_id: item.workspace_id,
        run_id: item.run_id,
        workspace_task_id: item.workspace_task_id,
        run_task_id: item.run_task_id,
        user_id: item.user_id,
        artifact_type: item.artifact_type,
        title: item.title,
        content: item.content,
        preview: item.preview,
        mime_type: item.mime_type,
        format: item.format,
        storage_kind: item.storage_kind,
        bucket: item.bucket,
        object_key: item.object_key,
        file_name: item.file_name,
        file_size: item.file_size,
        sha256: item.sha256,
        source: item.source,
        status: item.status,
        validation_status: item.validation_status,
        validation_report: item.validation_report,
        metadata: item.metadata,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
