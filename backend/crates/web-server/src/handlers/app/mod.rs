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

pub async fn list_conversations(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<ConversationResponse>> {
    let api = ConversationApi::new(get_app_ctx());
    let conversations: Vec<ConversationResponse> = api
        .list_conversations(auth_user.user_id)
        .await
        .map_err(from_biz_error)?
        .into_iter()
        .map(map_conversation_response)
        .collect();

    Ok(conversations.into_common_response().to_json())
}

pub async fn create_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateConversationRequest>,
) -> ResponseResult<ConversationDetailResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let detail = api
        .create_conversation(service::dto::conversation::CreateConversationRequest {
            user_id: auth_user.user_id,
            title: req.title,
            first_message: req.first_message,
            agent_id: req.agent_id,
            model_name: None,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_detail_response(detail)
        .into_common_response()
        .to_json())
}

pub async fn get_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
) -> ResponseResult<ConversationDetailResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let detail = api
        .get_conversation(auth_user.user_id, conversation_id)
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_detail_response(detail)
        .into_common_response()
        .to_json())
}

pub async fn add_conversation_message(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<AddConversationMessageRequest>,
) -> ResponseResult<ConversationMessageResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let message = api
        .add_message(service::dto::conversation::AddConversationMessageRequest {
            user_id: auth_user.user_id,
            conversation_id,
            role: req.role,
            content_text: req.content_text,
            content_json: req.content_json,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_message_response(message)
        .into_common_response()
        .to_json())
}

pub async fn update_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<UpdateConversationRequest>,
) -> ResponseResult<ConversationResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let conversation = api
        .update_conversation(service::dto::conversation::UpdateConversationRequest {
            user_id: auth_user.user_id,
            conversation_id,
            title: req.title,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_conversation_response(conversation)
        .into_common_response()
        .to_json())
}

pub async fn delete_conversation(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
) -> ResponseResult<()> {
    let api = ConversationApi::new(get_app_ctx());
    api.delete_conversation(service::dto::conversation::DeleteConversationRequest {
        user_id: auth_user.user_id,
        conversation_id,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn presign_conversation_file_upload(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<PresignConversationFileUploadRequest>,
) -> ResponseResult<PresignConversationFileUploadResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    api.get_conversation(auth_user.user_id.clone(), conversation_id.clone())
        .await
        .map_err(from_biz_error)?;

    let storage = get_storage();
    let ext = FileKind::ChatAttachment
        .normalize_ext(&req.extension)
        .map_err(|err| Error::Custom(err.to_string()))?;
    let content_type = req.content_type.or_else(|| {
        Some(
            FileKind::ChatAttachment
                .content_type_for_ext(&ext)
                .to_string(),
        )
    });
    let user_scope = storage.user_scope_key(&auth_user.user_id);
    let object_key = storage::key::chat_attachment_key(&user_scope, &conversation_id, &ext)
        .map_err(|err| Error::Custom(err.to_string()))?;
    let signed = storage
        .presign_upload(PresignUploadRequest {
            bucket: BucketKind::Private,
            key: object_key,
            content_type,
            expires_secs: Some(FILE_UPLOAD_EXPIRES_SECS),
        })
        .map_err(|err| Error::Custom(err.to_string()))?;

    Ok(PresignConversationFileUploadResponse {
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

pub async fn register_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<RegisterConversationFileRequest>,
) -> ResponseResult<ConversationFileResponse> {
    req.validate().map_err(Error::from)?;
    let storage = get_storage();
    let user_scope = storage.user_scope_key(&auth_user.user_id);
    let expected_prefix = format!("chat/{user_scope}/{conversation_id}/");
    if !req.object_key.starts_with(&expected_prefix) {
        return Err(Error::Custom("invalid conversation file object key".to_string()).into());
    }
    let api = ConversationApi::new(get_app_ctx());
    let file = api
        .register_file(
            service::dto::conversation::RegisterConversationFileRequest {
                user_id: auth_user.user_id,
                conversation_id,
                message_id: req.message_id,
                object_key: req.object_key,
                file_name: req.file_name,
                mime_type: req.mime_type,
                file_size: req.file_size,
                sha256: req.sha256,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_file_response(file).into_common_response().to_json())
}

pub async fn get_conversation_file_access(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<ConversationFileAccessResponse> {
    let api = ConversationApi::new(get_app_ctx());
    let access = api
        .file_access(get_storage(), auth_user.user_id, file_id)
        .await
        .map_err(from_biz_error)?;

    Ok(ConversationFileAccessResponse {
        file: map_file_response(access.file),
        method: access.method,
        download_url: access.download_url,
        expires_secs: access.expires_secs,
    }
    .into_common_response()
    .to_json())
}

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

pub async fn list_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListExecutionQueueQuery>,
) -> ResponseResult<ExecutionQueueListResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .list_execution_queue(service::dto::workspace::ListExecutionQueueRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            action_type: req.action_type,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_list_response(response)
        .into_common_response()
        .to_json())
}

pub async fn create_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .create_execution_queue(service::dto::workspace::CreateExecutionQueueRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            workspace_task_id: req.workspace_task_id,
            action_type: req.action_type,
            description: req.description,
            platform: req.platform,
            payload: req.payload,
            expected_impact: req.expected_impact,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn get_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
) -> ResponseResult<ExecutionQueueResponse> {
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .get_execution_queue(service::dto::workspace::GetExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn approve_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
    Json(req): Json<DecideExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .approve_execution_queue(service::dto::workspace::DecideExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn reject_execution_queue(
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
    Json(req): Json<DecideExecutionQueueRequest>,
) -> ResponseResult<ExecutionQueueResponse> {
    req.validate().map_err(Error::from)?;
    let api = WorkspaceApi::new(get_app_ctx());
    let response = api
        .reject_execution_queue(service::dto::workspace::DecideExecutionQueueRequest {
            user_id: auth_user.user_id,
            execution_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(map_execution_queue_response(response)
        .into_common_response()
        .to_json())
}

pub async fn list_tool_approvals(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListToolApprovalsQuery>,
) -> ResponseResult<ToolApprovalListResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .list_tool_approvals(service::dto::approval_audit::ListToolApprovalsRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            skill_name: req.skill_name,
            risk_level: req.risk_level,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .create_tool_approval(service::dto::approval_audit::CreateToolApprovalRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            conversation_id: req.conversation_id,
            request_role: req.request_role,
            skill_name: req.skill_name,
            tool_call_id: req.tool_call_id,
            risk_level: req.risk_level,
            args: req.args,
            action_key: req.action_key,
            reason: req.reason,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
) -> ResponseResult<ToolApprovalResponse> {
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .get_tool_approval(service::dto::approval_audit::GetToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn approve_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
    Json(req): Json<DecideToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .approve_tool_approval(service::dto::approval_audit::DecideToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn reject_tool_approval(
    Extension(auth_user): Extension<AuthUser>,
    Path(approval_id): Path<String>,
    Json(req): Json<DecideToolApprovalRequest>,
) -> ResponseResult<ToolApprovalResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .reject_tool_approval(service::dto::approval_audit::DecideToolApprovalRequest {
            user_id: auth_user.user_id,
            approval_id,
            reason: req.reason,
            result: req.result,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_audit_logs(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListAuditLogsQuery>,
) -> ResponseResult<AuditLogListResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .list_audit_logs(service::dto::approval_audit::ListAuditLogsRequest {
            user_id: auth_user.user_id,
            actor_user_id: req.actor_user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            execution_queue_id: req.execution_queue_id,
            tool_approval_id: req.tool_approval_id,
            action: req.action,
            target_type: req.target_type,
            status: req.status,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_audit_log(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAuditLogRequest>,
) -> ResponseResult<AuditLogResponse> {
    req.validate().map_err(Error::from)?;
    let api = ApprovalAuditApi::new(get_app_ctx());
    let response = api
        .create_audit_log(service::dto::approval_audit::CreateAuditLogRequest {
            user_id: auth_user.user_id,
            actor_user_id: req.actor_user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            execution_queue_id: req.execution_queue_id,
            tool_approval_id: req.tool_approval_id,
            action: req.action,
            target_type: req.target_type,
            target_id: req.target_id,
            status: req.status,
            reason: req.reason,
            request_id: req.request_id,
            trace_id: req.trace_id,
            metadata: req.metadata,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn list_long_tasks(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListLongTasksQuery>,
) -> ResponseResult<LongTaskListResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .list_long_tasks(service::dto::briefing_ops::ListLongTasksRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            status: req.status,
            task_type: req.task_type,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn create_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .create_long_task(service::dto::briefing_ops::CreateLongTaskRequest {
            user_id: auth_user.user_id,
            workspace_id: req.workspace_id,
            run_id: req.run_id,
            task_type: req.task_type,
            title: req.title,
            status: req.status,
            progress: req.progress,
            total_steps: req.total_steps,
            current_step: req.current_step,
            role: req.role,
            input_params: req.input_params,
            result: req.result,
            error_message: req.error_message,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
) -> ResponseResult<LongTaskResponse> {
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .get_long_task(service::dto::briefing_ops::GetLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
    Json(req): Json<UpdateLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .update_long_task(service::dto::briefing_ops::UpdateLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
            status: req.status,
            title: req.title,
            progress: req.progress,
            total_steps: req.total_steps,
            current_step: req.current_step,
            role: req.role,
            result: req.result,
            error_message: req.error_message,
            started_at: req.started_at,
            completed_at: req.completed_at,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn cancel_long_task(
    Extension(auth_user): Extension<AuthUser>,
    Path(task_id): Path<String>,
    Json(req): Json<CancelLongTaskRequest>,
) -> ResponseResult<LongTaskResponse> {
    req.validate().map_err(Error::from)?;
    let api = BriefingOpsApi::new(get_app_ctx());
    let response = api
        .cancel_long_task(service::dto::briefing_ops::CancelLongTaskRequest {
            user_id: auth_user.user_id,
            task_id,
            error_message: req.error_message,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

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

pub async fn list_platform_catalog() -> ResponseResult<PlatformCatalogResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    Ok(api.platform_catalog().into_common_response().to_json())
}

pub async fn list_platform_connections(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListPlatformConnectionsQuery>,
) -> ResponseResult<PlatformConnectionListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_platform_connections(
            service::dto::platform_connect::ListPlatformConnectionsRequest {
                user_id: auth_user.user_id,
                platform: req.platform,
                enabled: req.enabled,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SavePlatformConnectionRequest>,
) -> ResponseResult<PlatformConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_platform_connection(
            service::dto::platform_connect::SavePlatformConnectionRequest {
                user_id: auth_user.user_id,
                platform: req.platform,
                app_key: req.app_key,
                app_secret: req.app_secret,
                access_token: req.access_token,
                refresh_token: req.refresh_token,
                token_expires_at: req.token_expires_at,
                shop_id: req.shop_id,
                shop_name: req.shop_name,
                enabled: req.enabled,
                test_result: req.test_result,
                last_synced_at: req.last_synced_at,
                sync_status: req.sync_status,
                sync_message: req.sync_message,
                extra: req.extra,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<PlatformConnectionResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .get_platform_connection(
            service::dto::platform_connect::GetPlatformConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
    Json(req): Json<UpdatePlatformConnectionRequest>,
) -> ResponseResult<PlatformConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .update_platform_connection(
            service::dto::platform_connect::UpdatePlatformConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
                platform: req.platform,
                app_key: req.app_key,
                app_secret: req.app_secret,
                access_token: req.access_token,
                refresh_token: req.refresh_token,
                token_expires_at: req.token_expires_at,
                shop_id: req.shop_id,
                shop_name: req.shop_name,
                enabled: req.enabled,
                test_result: req.test_result,
                last_synced_at: req.last_synced_at,
                sync_status: req.sync_status,
                sync_message: req.sync_message,
                extra: req.extra,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn disable_platform_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<()> {
    let api = PlatformConnectApi::new(get_app_ctx());
    api.disable_platform_connection(
        service::dto::platform_connect::DisablePlatformConnectionRequest {
            user_id: auth_user.user_id,
            connection_id,
        },
    )
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_channel_catalog() -> ResponseResult<ChannelCatalogResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    Ok(api.channel_catalog().into_common_response().to_json())
}

pub async fn list_channel_connections(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListChannelConnectionsQuery>,
) -> ResponseResult<ChannelConnectionListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_channel_connections(
            service::dto::platform_connect::ListChannelConnectionsRequest {
                user_id: auth_user.user_id,
                channel: req.channel,
                enabled: req.enabled,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SaveChannelConnectionRequest>,
) -> ResponseResult<ChannelConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_channel_connection(
            service::dto::platform_connect::SaveChannelConnectionRequest {
                user_id: auth_user.user_id,
                channel: req.channel,
                webhook_url: req.webhook_url,
                bot_token: req.bot_token,
                chat_id: req.chat_id,
                secret: req.secret,
                extra_credentials: req.extra_credentials,
                enabled: req.enabled,
                last_error: req.last_error,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<ChannelConnectionResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .get_channel_connection(
            service::dto::platform_connect::GetChannelConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
    Json(req): Json<UpdateChannelConnectionRequest>,
) -> ResponseResult<ChannelConnectionResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .update_channel_connection(
            service::dto::platform_connect::UpdateChannelConnectionRequest {
                user_id: auth_user.user_id,
                connection_id,
                channel: req.channel,
                webhook_url: req.webhook_url,
                bot_token: req.bot_token,
                chat_id: req.chat_id,
                secret: req.secret,
                extra_credentials: req.extra_credentials,
                enabled: req.enabled,
                last_error: req.last_error,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn disable_channel_connection(
    Extension(auth_user): Extension<AuthUser>,
    Path(connection_id): Path<String>,
) -> ResponseResult<()> {
    let api = PlatformConnectApi::new(get_app_ctx());
    api.disable_channel_connection(
        service::dto::platform_connect::DisableChannelConnectionRequest {
            user_id: auth_user.user_id,
            connection_id,
        },
    )
    .await
    .map_err(from_biz_error)?;
    Ok(().into_common_response().to_json())
}

pub async fn list_store_metrics(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ListStoreMetricsQuery>,
) -> ResponseResult<StoreMetricListResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .list_metrics(service::dto::platform_connect::ListStoreMetricsRequest {
            user_id: auth_user.user_id,
            platform: req.platform,
            metric_key: req.metric_key,
            metric_date_from: req.metric_date_from,
            metric_date_to: req.metric_date_to,
            source: req.source,
            page: req.page,
            page_size: req.page_size,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn save_store_metric(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SaveStoreMetricRequest>,
) -> ResponseResult<StoreMetricResponse> {
    req.validate().map_err(Error::from)?;
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .save_metric(service::dto::platform_connect::SaveStoreMetricRequest {
            user_id: auth_user.user_id,
            platform: req.platform,
            metric_date: req.metric_date,
            metric_key: req.metric_key,
            metric_value: req.metric_value,
            dimension_key: req.dimension_key,
            dimension_value: req.dimension_value,
            source: req.source,
        })
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn platform_sync_status(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<PlatformSyncStatusListResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .sync_status(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn platform_summary(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<PlatformSummaryResponse> {
    let api = PlatformConnectApi::new(get_app_ctx());
    let response = api
        .summary(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

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

fn map_conversation_detail_response(
    detail: service::dto::conversation::ConversationDetailResponse,
) -> ConversationDetailResponse {
    ConversationDetailResponse {
        conversation: map_conversation_response(detail.conversation),
        messages: detail
            .messages
            .into_iter()
            .map(map_message_response)
            .collect(),
        files: detail.files.into_iter().map(map_file_response).collect(),
    }
}

fn map_conversation_response(
    item: service::dto::conversation::ConversationResponse,
) -> ConversationResponse {
    ConversationResponse {
        id: item.id,
        user_id: item.user_id,
        title: item.title,
        status: item.status,
        source: item.source,
        agent_id: item.agent_id,
        model_name: item.model_name,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_message_response(
    item: service::dto::conversation::ConversationMessageResponse,
) -> ConversationMessageResponse {
    ConversationMessageResponse {
        id: item.id,
        conversation_id: item.conversation_id,
        user_id: item.user_id,
        role: item.role,
        content_text: item.content_text,
        content_json: item.content_json,
        status: item.status,
        sequence: item.sequence,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_file_response(
    item: service::dto::conversation::ConversationFileResponse,
) -> ConversationFileResponse {
    ConversationFileResponse {
        id: item.id,
        conversation_id: item.conversation_id,
        message_id: item.message_id,
        bucket: item.bucket,
        object_key: item.object_key,
        file_name: item.file_name,
        mime_type: item.mime_type,
        file_size: item.file_size,
        sha256: item.sha256,
        status: item.status,
        created_at: item.created_at,
    }
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

fn map_execution_queue_list_response(
    item: service::dto::workspace::ExecutionQueueListResponse,
) -> ExecutionQueueListResponse {
    ExecutionQueueListResponse {
        items: item
            .items
            .into_iter()
            .map(map_execution_queue_response)
            .collect(),
        total: item.total,
        page: item.page,
        page_size: item.page_size,
    }
}

fn map_execution_queue_response(
    item: service::dto::workspace::ExecutionQueueResponse,
) -> ExecutionQueueResponse {
    ExecutionQueueResponse {
        id: item.id,
        user_id: item.user_id,
        workspace_id: item.workspace_id,
        run_id: item.run_id,
        workspace_task_id: item.workspace_task_id,
        action_type: item.action_type,
        description: item.description,
        platform: item.platform,
        payload: item.payload,
        expected_impact: item.expected_impact,
        status: item.status,
        decision_reason: item.decision_reason,
        result: item.result,
        created_at: item.created_at,
        decided_at: item.decided_at,
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
