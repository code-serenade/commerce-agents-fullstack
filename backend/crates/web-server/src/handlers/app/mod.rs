use axum::{Extension, Json, extract::Path};
use service::api::{app::AppApi, conversation::ConversationApi};
use storage::{BucketKind, FileKind, PresignUploadRequest};
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::app::{
        AddConversationMessageRequest, AgentLlmConfigItemResponse, AgentLlmConfigResponse,
        AppUserResponse, ConversationDetailResponse, ConversationFileAccessResponse,
        ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
        CreateConversationRequest, CurrentUserPermissionsResponse,
        PresignConversationFileUploadRequest, PresignConversationFileUploadResponse,
        RegisterAppUserRequest, RegisterConversationFileRequest,
    },
    error::{Error, from_biz_error},
    statics::{
        agent_manager::get_agent_runtime_config, db_manager::get_app_ctx,
        storage_manager::get_storage,
    },
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

pub async fn agent_llm_config() -> ResponseResult<AgentLlmConfigResponse> {
    let config = get_agent_runtime_config().map_err(|err| Error::Custom(err.to_string()))?;
    let selected_name = config
        .default_llm
        .as_ref()
        .or_else(|| config.llm.first().map(|llm| &llm.name));
    let items = config
        .llm
        .iter()
        .map(|llm| AgentLlmConfigItemResponse {
            name: llm.name.clone(),
            base_url: llm.base_url.clone(),
            model: llm.model.clone(),
            max_tokens: llm.max_tokens,
            temperature: llm.temperature,
            reasoning_effort: llm.reasoning_effort.clone(),
            chat_completions_endpoint: llm.chat_completions_endpoint.clone(),
            has_api_key: llm.api_key.as_deref().is_some_and(|key| !key.is_empty()),
            selected: selected_name.is_some_and(|name| name == &llm.name),
        })
        .collect();

    Ok(AgentLlmConfigResponse {
        default_llm: config.default_llm.clone(),
        selected_llm: selected_name.cloned(),
        items,
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
            model_name: req.model_name,
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
