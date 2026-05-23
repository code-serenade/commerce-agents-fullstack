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
    vision_preprocess_manager::{
        EnsureVisionPreprocessRequest, VisionPreprocessConfig, VisionPreprocessManager,
    },
    workspace::WorkspaceApi,
};
use storage::{BucketKind, DownloadObjectRequest, PresignDownloadRequest};
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use utils::image::vision_image_data_url;
use uuid::Uuid;
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
        UpdateConversationFileExtractionRequest, UpdateConversationRequest, UpdateLongTaskRequest,
        UpdatePackageRequest, UpdatePlatformConnectionRequest, UpdateProductAssetRequest,
        UpdateProductCompetitorRequest, UpdateProductKnowledgeRequest,
        UpdateProductMaterialRequest, UpdateProductRequest, UpdateWorkspaceRequest,
        UpdateWorkspaceRunRequest, UpdateWorkspaceRunTaskRequest, UpdateWorkspaceTaskRequest,
        UpsertDailyBriefingRecordRequest, UpsertPackageRequest, WorkspaceListResponse,
        WorkspaceResponse, WorkspaceRunListResponse, WorkspaceRunResponse,
        WorkspaceRunTaskResponse, WorkspaceTaskListResponse, WorkspaceTaskResponse,
    },
    error::{Error, from_biz_error},
    statics::{
        agent_manager::{agent_runtime_llm_usage, run_vision_preprocess, vision_preprocess_config},
        db_manager::get_app_ctx,
        storage_manager::get_storage,
    },
};

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
    let _ = (auth_user, conversation_id, req);
    Err(
        Error::Custom("conversation file upload signing has moved to file-service".to_string())
            .into(),
    )
}

pub async fn register_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    Json(req): Json<RegisterConversationFileRequest>,
) -> ResponseResult<ConversationFileResponse> {
    tracing::debug!(
        user_id = %auth_user.user_id,
        conversation_id = %conversation_id,
        object_key = %req.object_key,
        mime_type = ?req.mime_type,
        file_size = ?req.file_size,
        "register conversation file request received"
    );
    req.validate().map_err(Error::from)?;
    let user_id_for_log = auth_user.user_id.clone();
    let user_id_for_extraction = auth_user.user_id.clone();
    let conversation_id_for_log = conversation_id.clone();
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
    tracing::debug!(
        user_id = %user_id_for_log,
        conversation_id = %conversation_id_for_log,
        file_id = %file.id,
        object_key = %file.object_key,
        mime_type = ?file.mime_type,
        status = %file.status,
        "register conversation file succeeded"
    );

    spawn_vision_preprocess_if_needed(&file);
    spawn_document_text_extraction_if_needed(&user_id_for_extraction, &file);

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

pub async fn delete_conversation_file(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
) -> ResponseResult<()> {
    let api = ConversationApi::new(get_app_ctx());
    api.delete_file(
        get_storage(),
        service::dto::conversation::DeleteConversationFileRequest {
            user_id: auth_user.user_id,
            file_id,
        },
    )
    .await
    .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn update_conversation_file_extraction(
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<String>,
    Json(req): Json<UpdateConversationFileExtractionRequest>,
) -> ResponseResult<ConversationFileResponse> {
    req.validate().map_err(Error::from)?;
    let api = ConversationApi::new(get_app_ctx());
    let file = api
        .update_file_extraction(
            service::dto::conversation::UpdateConversationFileExtractionRequest {
                user_id: auth_user.user_id,
                file_id,
                status: req.status,
                text: req.text,
                content_kind: req.content_kind,
                source: req.source,
                parser: req.parser,
                language: req.language,
                failure_reason: req.failure_reason,
                metadata: req.metadata,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_file_response(file).into_common_response().to_json())
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
        metadata: item.metadata,
        created_at: item.created_at,
    }
}

fn spawn_vision_preprocess_if_needed(file: &service::dto::conversation::ConversationFileResponse) {
    let Ok(conversation_file_id) = Uuid::parse_str(&file.id) else {
        tracing::warn!(file_id = %file.id, "vision preprocess skipped: invalid file id");
        return;
    };

    let Ok(agent_config) = vision_preprocess_config() else {
        tracing::warn!("vision preprocess skipped: config unavailable");
        return;
    };
    let Ok(llm_usage) = agent_runtime_llm_usage() else {
        tracing::warn!("vision preprocess skipped: llm usage unavailable");
        return;
    };
    let Ok(llm_name) = agent_config.usage_name(&llm_usage) else {
        tracing::warn!("vision preprocess skipped: configured llm usage unavailable");
        return;
    };

    let config = VisionPreprocessConfig {
        analysis_kind: agent_config.analysis_kind.clone(),
        prompt_key: agent_config.prompt.clone(),
        prompt_version: agent_config.prompt_version.clone(),
        llm_name,
        llm_model: None,
    };
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        conversation_id = %file.conversation_id,
        object_key = %file.object_key,
        mime_type = ?file.mime_type,
        file_size = ?file.file_size,
        analysis_kind = %config.analysis_kind,
        prompt_key = %config.prompt_key,
        prompt_version = %config.prompt_version,
        llm_name = %config.llm_name,
        "vision preprocess task spawned"
    );

    tokio::spawn(async move {
        if let Err(err) = run_vision_preprocess_task(conversation_file_id, config).await {
            tracing::warn!(
                conversation_file_id = %conversation_file_id,
                error = %err,
                "vision preprocess task failed"
            );
        }
    });
}

fn spawn_document_text_extraction_if_needed(
    user_id: &str,
    file: &service::dto::conversation::ConversationFileResponse,
) {
    if is_supported_text_document_mime(file.mime_type.as_deref())
        || file
            .file_name
            .as_deref()
            .is_some_and(is_supported_text_document_name)
        || is_supported_text_document_name(&file.object_key)
    {
        let user_id = user_id.to_string();
        let file_id = file.id.clone();
        tracing::debug!(
            file_id = %file_id,
            object_key = %file.object_key,
            mime_type = ?file.mime_type,
            "document text extraction task spawned"
        );
        tokio::spawn(async move {
            let api = ConversationApi::new(get_app_ctx());
            if let Err(err) = api
                .extract_file_text(get_storage(), user_id, file_id.clone())
                .await
            {
                tracing::warn!(
                    file_id = %file_id,
                    error = ?err,
                    "document text extraction task failed"
                );
            }
        });
    }
}

fn is_supported_text_document_mime(value: Option<&str>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        mime.as_str(),
        "application/pdf"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel"
            | "text/csv"
            | "text/plain"
            | "text/markdown"
            | "application/json"
    )
}

fn is_supported_text_document_name(value: &str) -> bool {
    let extension = value
        .rsplit_once('.')
        .map(|(_, extension)| extension.trim().to_ascii_lowercase());
    matches!(
        extension.as_deref(),
        Some(
            "pdf"
                | "docx"
                | "xlsx"
                | "xlsm"
                | "xlsb"
                | "xls"
                | "csv"
                | "txt"
                | "md"
                | "markdown"
                | "json"
        )
    )
}

async fn run_vision_preprocess_task(
    conversation_file_id: Uuid,
    config: VisionPreprocessConfig,
) -> Result<(), Error> {
    use service::api::storage_object_analysis::{
        CompleteStorageObjectAnalysisRequest, StorageObjectAnalysisApi,
    };

    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        "vision preprocess task preparing"
    );
    let manager = VisionPreprocessManager::new(get_app_ctx());
    let Some(task) = manager
        .ensure_task_for_conversation_file(EnsureVisionPreprocessRequest {
            conversation_file_id,
            config,
        })
        .await
        .map_err(|err| Error::Custom(format!("vision preprocess task ensure failed: {err:?}")))?
    else {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            "vision preprocess task skipped: unsupported or inactive file"
        );
        return Ok(());
    };
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        storage_object_sha256 = %task.object.sha256,
        analysis_id = %task.analysis.id,
        sha256 = %task.object.sha256,
        current_status = %task.analysis.status,
        "vision preprocess task loaded"
    );

    if !StorageObjectAnalysisApi::can_process(&task.analysis.status) {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            status = %task.analysis.status,
            "vision preprocess task skipped: analysis already processed or running"
        );
        return Ok(());
    }

    let analysis_api = StorageObjectAnalysisApi::new(get_app_ctx());
    analysis_api
        .mark_processing(task.analysis.id, task.analysis.llm_model.clone())
        .await
        .map_err(|err| {
            Error::Custom(format!("vision preprocess mark processing failed: {err:?}"))
        })?;
    tracing::debug!(
        conversation_file_id = %conversation_file_id,
        analysis_id = %task.analysis.id,
        "vision preprocess marked processing"
    );

    let mime_type = task
        .file
        .mime_type
        .clone()
        .unwrap_or_else(|| "image/jpeg".to_string());
    let result = async {
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            object_key = %task.file.object_key,
            bucket = %task.file.bucket,
            "vision preprocess image download starting"
        );
        let bytes = get_storage()
            .download_object(DownloadObjectRequest {
                bucket: BucketKind::Private,
                key: task.file.object_key.clone(),
            })
            .await
            .map_err(|err| Error::Custom(format!("vision preprocess download failed: {err}")))?;
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            original_bytes = bytes.len(),
            mime_type = %mime_type,
            "vision preprocess image download finished"
        );
        let data_url = vision_image_data_url(&mime_type, &bytes)
            .map_err(|err| Error::Custom(format!("vision preprocess data url failed: {err}")))?;
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            data_url_bytes = data_url.len(),
            "vision preprocess image prepared"
        );
        tracing::debug!(
            conversation_file_id = %conversation_file_id,
            analysis_id = %task.analysis.id,
            "vision preprocess agent starting"
        );
        run_vision_preprocess(agent::api::VisionPreprocessInput {
            data_url,
            file_name: task.file.file_name.clone(),
            mime_type: task.file.mime_type.clone(),
            object_key: task.file.object_key.clone(),
            bucket: task.file.bucket.clone(),
            file_size: task.file.file_size,
        })
        .await
    }
    .await;

    match result {
        Ok(output) => {
            analysis_api
                .mark_succeeded(CompleteStorageObjectAnalysisRequest {
                    id: task.analysis.id,
                    llm_model: output.llm_model,
                    summary: output.summary,
                    result_json: output.result_json,
                })
                .await
                .map_err(|err| {
                    Error::Custom(format!("vision preprocess mark succeeded failed: {err:?}"))
                })?;
            tracing::debug!(
                conversation_file_id = %conversation_file_id,
                analysis_id = %task.analysis.id,
                "vision preprocess task succeeded"
            );
        }
        Err(err) => {
            analysis_api
                .delete(task.analysis.id)
                .await
                .map_err(|delete_err| {
                    Error::Custom(format!(
                        "vision preprocess delete failed analysis failed: {delete_err:?}"
                    ))
                })?;
            return Err(err);
        }
    }

    Ok(())
}
