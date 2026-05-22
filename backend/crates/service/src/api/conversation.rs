use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    conversation_files::{
        ConversationFile, ConversationFileService, CreateConversationFile,
        UpdateConversationFileMetadata, UpdateConversationFileStatus,
    },
    conversation_messages::{
        ConversationMessage, ConversationMessageService, CreateConversationMessage,
    },
    conversations::{Conversation, ConversationService, CreateConversation},
};
use serde_json::{Map, Value, json};
use storage::{BucketKind, DownloadObjectRequest, ObjectStorage, PresignDownloadRequest};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    api::storage_reference::{RetainStorageReferenceRequest, StorageReferenceApi},
    dto::conversation::{
        AddConversationMessageRequest, ConversationDetailResponse, ConversationFileAccessResponse,
        ConversationFileExtractionJobResponse, ConversationFileExtractionStatusResponse,
        ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
        CreateConversationRequest, DeleteConversationFileRequest, DeleteConversationRequest,
        RegisterConversationFileRequest, TriggerConversationFileExtractionJobRequest,
        UpdateConversationFileExtractionRequest, UpdateConversationMemoryRequest,
        UpdateConversationRequest, uuid_to_string,
    },
};

const DEFAULT_LIST_LIMIT: u64 = 50;
const FILE_ACCESS_EXPIRES_SECS: u64 = 600;
const MAX_SERVER_TEXT_OBJECT_BYTES: usize = 1_000_000;
const MAX_EXTRACTED_TEXT_CHARS: usize = 100_000;
const EXTRACTION_STATUS_PENDING: &str = "pending";
const EXTRACTION_STATUS_EXTRACTED: &str = "extracted";
const EXTRACTION_STATUS_FAILED: &str = "failed";
const EXTRACTION_STATUS_UNSUPPORTED: &str = "unsupported";

pub struct ConversationApi {
    db: DbContext,
    app_user_svc: AppUserService,
    conversation_svc: ConversationService,
    message_svc: ConversationMessageService,
    file_svc: ConversationFileService,
}

impl ConversationApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            app_user_svc: AppUserService::new(db.clone()),
            conversation_svc: ConversationService::new(db.clone()),
            message_svc: ConversationMessageService::new(db.clone()),
            file_svc: ConversationFileService::new(db),
        }
    }

    pub async fn list_conversations(
        &self,
        user_id: String,
    ) -> BizResult<Vec<ConversationResponse>> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        Ok(self
            .conversation_svc
            .list_by_user_id(user_id, DEFAULT_LIST_LIMIT)
            .await?
            .into_iter()
            .map(Self::map_conversation)
            .collect())
    }

    pub async fn create_conversation(
        &self,
        req: CreateConversationRequest,
    ) -> BizResult<ConversationDetailResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;

        let tx = self.db.begin().await?;
        let result: BizResult<ConversationDetailResponse> = async {
            let api = ConversationApi::new(tx.clone());
            let conversation = api
                .conversation_svc
                .create(CreateConversation {
                    id: Uuid::new_v4(),
                    user_id,
                    title: normalize_optional(req.title),
                    source: "web".to_string(),
                    agent_id: normalize_optional(req.agent_id),
                    model_name: normalize_optional(req.model_name),
                    metadata: json!({}),
                })
                .await?;

            let mut messages = Vec::new();
            if let Some(first_message) = normalize_optional(req.first_message) {
                messages.push(
                    api.message_svc
                        .create(CreateConversationMessage {
                            id: Uuid::new_v4(),
                            conversation_id: conversation.id,
                            user_id,
                            role: "user".to_string(),
                            content_text: Some(first_message),
                            content_json: None,
                            sequence: 1,
                            token_count: None,
                            metadata: json!({}),
                        })
                        .await?,
                );
            }

            Ok(ConversationDetailResponse {
                conversation: Self::map_conversation(conversation),
                messages: messages.into_iter().map(Self::map_message).collect(),
                files: Vec::new(),
            })
        }
        .await;
        commit_or_rollback(tx, result).await
    }

    pub async fn get_conversation(
        &self,
        user_id: String,
        conversation_id: String,
    ) -> BizResult<ConversationDetailResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_conversation_owner(user_id, conversation_id)
            .await?;
        let messages = self
            .message_svc
            .list_by_conversation_id(conversation_id)
            .await?;
        let files = self
            .file_svc
            .list_by_conversation_id(conversation_id)
            .await?;

        Ok(ConversationDetailResponse {
            conversation: Self::map_conversation(conversation),
            messages: messages.into_iter().map(Self::map_message).collect(),
            files: files.into_iter().map(Self::map_file).collect(),
        })
    }

    pub async fn add_message(
        &self,
        req: AddConversationMessageRequest,
    ) -> BizResult<ConversationMessageResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        self.ensure_conversation_owner(user_id, conversation_id)
            .await?;
        let role = normalize_role(&req.role)?;
        let content_text = normalize_optional(req.content_text);
        let content_json = req.content_json;
        if content_text.is_none() && content_json.is_none() {
            return Err(BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                "message content is empty".to_string(),
            ));
        }

        let sequence = self.message_svc.next_sequence(conversation_id).await?;
        let message = self
            .message_svc
            .create(CreateConversationMessage {
                id: Uuid::new_v4(),
                conversation_id,
                user_id,
                role,
                content_text,
                content_json,
                sequence,
                token_count: None,
                metadata: json!({}),
            })
            .await?;
        Ok(Self::map_message(message))
    }

    pub async fn update_conversation(
        &self,
        req: UpdateConversationRequest,
    ) -> BizResult<ConversationResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_active_conversation_owner(user_id, conversation_id)
            .await?;

        let title = normalize_optional(req.title);
        let updated = self
            .conversation_svc
            .update(repo::table::conversations::UpdateConversation {
                id: conversation.id,
                title,
                status: conversation.status,
                archived_at: conversation.archived_at,
            })
            .await?;

        Ok(Self::map_conversation(updated))
    }

    pub async fn update_conversation_memory(
        &self,
        req: UpdateConversationMemoryRequest,
    ) -> BizResult<ConversationResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_active_conversation_owner(user_id, conversation_id)
            .await?;
        if !req.memory.is_object() {
            return Err(BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                "memory must be a JSON object".to_string(),
            ));
        }

        let mut metadata = conversation
            .metadata
            .as_object()
            .cloned()
            .unwrap_or_default();
        metadata.insert("memorySummary".to_string(), req.memory);
        metadata.insert("memoryUpdatedAt".to_string(), Value::String(now_rfc3339()));
        let updated = self
            .conversation_svc
            .update_metadata(repo::table::conversations::UpdateConversationMetadata {
                id: conversation.id,
                metadata: Value::Object(metadata),
            })
            .await?;

        Ok(Self::map_conversation(updated))
    }

    pub async fn get_conversation_memory(
        &self,
        user_id: String,
        conversation_id: String,
    ) -> BizResult<Option<Value>> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_active_conversation_owner(user_id, conversation_id)
            .await?;
        Ok(conversation
            .metadata
            .get("memorySummary")
            .or_else(|| conversation.metadata.get("memory_summary"))
            .filter(|value| value.is_object())
            .cloned())
    }

    pub async fn delete_conversation(&self, req: DeleteConversationRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        self.ensure_active_conversation_owner(user_id, conversation_id)
            .await?;

        let tx = self.db.begin().await?;
        let result: BizResult<()> = async {
            let api = ConversationApi::new(tx.clone());
            let storage_ref_api = StorageReferenceApi::new(tx.clone());
            let files = api
                .file_svc
                .list_by_conversation_id(conversation_id)
                .await?;
            for file in files {
                storage_ref_api
                    .release_in_current_context("conversation_file", file.id)
                    .await?;
            }
            api.conversation_svc.delete_by_id(conversation_id).await?;
            Ok(())
        }
        .await;

        commit_or_rollback(tx, result).await
    }

    pub async fn register_file(
        &self,
        req: RegisterConversationFileRequest,
    ) -> BizResult<ConversationFileResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        self.ensure_active_conversation_owner(user_id, conversation_id)
            .await?;
        let message_id = req
            .message_id
            .as_deref()
            .map(|value| parse_uuid(value, "message_id"))
            .transpose()?;

        let object_key = normalize_required_text(req.object_key, "object_key")?;
        ensure_file_service_object_key(&object_key, &user_id)?;
        if let Some(sha256) = normalize_optional(req.sha256.clone()) {
            ensure_object_key_sha256_matches(&object_key, &sha256)?;
        }

        if let Some(existing) = self
            .file_svc
            .get_active_by_conversation_object(conversation_id, "private", &object_key)
            .await?
        {
            StorageReferenceApi::new(self.db.clone())
                .retain(RetainStorageReferenceRequest {
                    user_id,
                    bucket: existing.bucket.clone(),
                    object_key: existing.object_key.clone(),
                    sha256: existing.sha256.clone(),
                    mime_type: existing.mime_type.clone(),
                    file_size: existing.file_size,
                    source_type: "conversation_file".to_string(),
                    source_id: existing.id,
                    metadata: json!({}),
                })
                .await?;
            return Ok(Self::map_file(existing));
        }

        let tx = self.db.begin().await?;
        let result: BizResult<ConversationFileResponse> = async {
            let api = ConversationApi::new(tx.clone());
            let file = api
                .file_svc
                .create(CreateConversationFile {
                    id: Uuid::new_v4(),
                    user_id,
                    conversation_id,
                    message_id,
                    bucket: "private".to_string(),
                    object_key,
                    file_name: normalize_optional(req.file_name),
                    mime_type: normalize_optional(req.mime_type),
                    file_size: req.file_size,
                    sha256: normalize_optional(req.sha256),
                    metadata: build_initial_file_extraction_metadata(),
                })
                .await?;

            StorageReferenceApi::new(tx.clone())
                .retain_in_current_context(RetainStorageReferenceRequest {
                    user_id,
                    bucket: file.bucket.clone(),
                    object_key: file.object_key.clone(),
                    sha256: file.sha256.clone(),
                    mime_type: file.mime_type.clone(),
                    file_size: file.file_size,
                    source_type: "conversation_file".to_string(),
                    source_id: file.id,
                    metadata: json!({}),
                })
                .await?;

            Ok(Self::map_file(file))
        }
        .await;

        commit_or_rollback(tx, result).await
    }

    pub async fn delete_file(
        &self,
        _storage: &ObjectStorage,
        req: DeleteConversationFileRequest,
    ) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;

        let tx = self.db.begin().await?;
        let result: BizResult<()> = async {
            let api = ConversationApi::new(tx.clone());
            StorageReferenceApi::new(tx.clone())
                .release_in_current_context("conversation_file", file.id)
                .await?;
            api.file_svc
                .update_status(UpdateConversationFileStatus {
                    id: file.id,
                    status: "deleted".to_string(),
                    deleted_at: Some(OffsetDateTime::now_utc()),
                })
                .await?;
            Ok(())
        }
        .await;

        commit_or_rollback(tx, result).await?;

        Ok(())
    }

    pub async fn file_access(
        &self,
        storage: &ObjectStorage,
        user_id: String,
        file_id: String,
    ) -> BizResult<ConversationFileAccessResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&file_id, "file_id")?;
        let file = self.file_svc.get_by_id(file_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::CONVERSATION_FILE_NOT_FOUND,
                "file not found".to_string(),
            )
        })?;
        if file.user_id != user_id {
            return Err(BizError::new(
                app_error::CONVERSATION_FILE_FORBIDDEN,
                "file does not belong to current user".to_string(),
            ));
        }

        let signed = storage
            .presign_download(PresignDownloadRequest {
                bucket: BucketKind::Private,
                key: file.object_key.clone(),
                expires_secs: Some(FILE_ACCESS_EXPIRES_SECS),
            })
            .map_err(|err| {
                BizError::new(app_error::CONVERSATION_REQUEST_INVALID, err.to_string())
            })?;

        Ok(ConversationFileAccessResponse {
            file: Self::map_file(file),
            method: signed.method,
            download_url: signed.download_url,
            expires_secs: FILE_ACCESS_EXPIRES_SECS,
        })
    }

    pub async fn update_file_extraction(
        &self,
        req: UpdateConversationFileExtractionRequest,
    ) -> BizResult<ConversationFileResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;

        let status = normalize_extraction_status(req.status)?;
        let text = if status == EXTRACTION_STATUS_EXTRACTED {
            let text = req
                .text
                .map(|text| normalize_required_text(text, "text"))
                .transpose()?
                .ok_or_else(|| {
                    BizError::new(
                        app_error::CONVERSATION_REQUEST_INVALID,
                        "text is required when extraction status is extracted".to_string(),
                    )
                })?;
            let text_chars = text.chars().count();
            if text_chars > MAX_EXTRACTED_TEXT_CHARS {
                return Err(BizError::new(
                    app_error::CONVERSATION_REQUEST_INVALID,
                    format!("text is too long: max {MAX_EXTRACTED_TEXT_CHARS} chars"),
                ));
            }
            Some(text)
        } else {
            None
        };
        let text_chars = text.as_deref().map(str::chars).map(Iterator::count);

        let content_kind =
            normalize_optional(req.content_kind).unwrap_or_else(|| "plain_text".into());
        let source = normalize_optional(req.source).unwrap_or_else(|| "trusted_api".into());
        let parser = normalize_optional(req.parser).unwrap_or_else(|| {
            if source == "manual" {
                "manual".into()
            } else {
                "external_adapter".into()
            }
        });
        let language = normalize_optional(req.language);
        let failure_reason = normalize_optional(req.failure_reason);
        let extra_metadata = req.metadata.unwrap_or_else(|| json!({}));
        if !extra_metadata.is_object() {
            return Err(BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                "metadata must be a JSON object".to_string(),
            ));
        }

        let metadata = build_file_extraction_metadata(
            file.metadata,
            FileExtractionMetadataPatch {
                status,
                text,
                content_kind,
                source,
                parser,
                language,
                failure_reason,
                text_chars: text_chars.unwrap_or(0),
                extra_metadata,
            },
        );
        let updated = self
            .file_svc
            .update_metadata(UpdateConversationFileMetadata {
                id: file.id,
                metadata,
            })
            .await?;

        Ok(Self::map_file(updated))
    }

    pub async fn get_file_extraction_status(
        &self,
        user_id: String,
        file_id: String,
    ) -> BizResult<ConversationFileExtractionStatusResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;
        Ok(build_file_extraction_status_response(&file))
    }

    pub async fn trigger_file_extraction_job(
        &self,
        storage: &ObjectStorage,
        req: TriggerConversationFileExtractionJobRequest,
    ) -> BizResult<ConversationFileExtractionJobResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;

        let current = build_file_extraction_status_response(&file);
        if current.readable && !req.force {
            return Ok(ConversationFileExtractionJobResponse {
                job_id: format!("noop-{}", file.id),
                file_id: uuid_to_string(file.id),
                status: current.status,
                content_kind: current.content_kind,
                parser: current.parser,
                source: current.source,
                message: "file already has readable extraction; no job was started".to_string(),
                metadata: json!({ "mode": req.mode.unwrap_or_else(|| "auto".to_string()) }),
            });
        }

        let mode = req.mode.unwrap_or_else(|| "auto".to_string());
        let (mime_type, file_name) = normalized_file_identity(&file);
        let plan = if is_server_text_extractable(&mime_type, &file_name) {
            build_server_text_extraction_plan(storage, &file).await
        } else if is_server_pdf_extractable(&mime_type, &file_name) {
            build_server_pdf_text_extraction_plan(storage, &file).await
        } else if is_server_ooxml_extractable(&mime_type, &file_name) {
            build_server_ooxml_extraction_plan(storage, &file).await
        } else {
            classify_server_extraction_plan(&file)
        };
        let metadata = build_file_extraction_metadata(
            file.metadata,
            FileExtractionMetadataPatch {
                status: plan.status.clone(),
                text: plan.text.clone(),
                content_kind: plan.content_kind.clone(),
                source: "server_job".to_string(),
                parser: plan.parser.clone(),
                language: None,
                failure_reason: plan.failure_reason.clone(),
                text_chars: plan
                    .text
                    .as_deref()
                    .map(str::chars)
                    .map(Iterator::count)
                    .unwrap_or(0),
                extra_metadata: build_server_extraction_extra_metadata(&plan, mode),
            },
        );
        let updated = self
            .file_svc
            .update_metadata(UpdateConversationFileMetadata {
                id: file.id,
                metadata,
            })
            .await?;
        let status = build_file_extraction_status_response(&updated);
        Ok(ConversationFileExtractionJobResponse {
            job_id: plan.job_id,
            file_id: status.file_id,
            status: status.status,
            content_kind: status.content_kind,
            parser: status.parser,
            source: status.source,
            message: plan.message,
            metadata: status.metadata,
        })
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_conversation_owner(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> BizResult<Conversation> {
        let conversation = self
            .conversation_svc
            .get_by_id(conversation_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::CONVERSATION_NOT_FOUND,
                    "conversation not found".to_string(),
                )
            })?;
        if conversation.user_id != user_id {
            return Err(BizError::new(
                app_error::CONVERSATION_FORBIDDEN,
                "conversation does not belong to current user".to_string(),
            ));
        }
        Ok(conversation)
    }

    async fn ensure_active_conversation_owner(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> BizResult<Conversation> {
        let conversation = self
            .ensure_conversation_owner(user_id, conversation_id)
            .await?;
        if conversation.status == "deleted" {
            return Err(BizError::new(
                app_error::CONVERSATION_NOT_FOUND,
                "conversation not found".to_string(),
            ));
        }
        Ok(conversation)
    }

    async fn ensure_active_file_owner(
        &self,
        user_id: Uuid,
        file_id: Uuid,
    ) -> BizResult<ConversationFile> {
        let file = self.file_svc.get_by_id(file_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::CONVERSATION_FILE_NOT_FOUND,
                "file not found".to_string(),
            )
        })?;
        if file.user_id != user_id {
            return Err(BizError::new(
                app_error::CONVERSATION_FILE_FORBIDDEN,
                "file does not belong to current user".to_string(),
            ));
        }
        if file.status != "active" {
            return Err(BizError::new(
                app_error::CONVERSATION_FILE_NOT_FOUND,
                "file not found".to_string(),
            ));
        }
        Ok(file)
    }

    fn map_conversation(item: Conversation) -> ConversationResponse {
        ConversationResponse {
            id: uuid_to_string(item.id),
            user_id: uuid_to_string(item.user_id),
            title: item.title,
            status: item.status,
            source: item.source,
            agent_id: item.agent_id,
            model_name: item.model_name,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_message(item: ConversationMessage) -> ConversationMessageResponse {
        ConversationMessageResponse {
            id: uuid_to_string(item.id),
            conversation_id: uuid_to_string(item.conversation_id),
            user_id: uuid_to_string(item.user_id),
            role: item.role,
            content_text: item.content_text,
            content_json: item.content_json,
            status: item.status,
            sequence: item.sequence,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_file(item: ConversationFile) -> ConversationFileResponse {
        ConversationFileResponse {
            id: uuid_to_string(item.id),
            conversation_id: uuid_to_string(item.conversation_id),
            message_id: item.message_id.map(uuid_to_string),
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
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

fn normalize_role(value: &str) -> BizResult<String> {
    match value {
        "user" | "assistant" | "system" | "tool" => Ok(value.to_string()),
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("invalid message role: {value}"),
        )),
    }
}

fn normalize_required_text(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("{label} is empty"),
        ));
    }
    Ok(value)
}

fn ensure_file_service_object_key(object_key: &str, user_id: &Uuid) -> BizResult<()> {
    storage::key::validate_object_key(object_key)
        .map_err(|err| BizError::new(app_error::CONVERSATION_REQUEST_INVALID, err.to_string()))?;

    let user_scope = file_service_user_scope_key(user_id);
    let docs_prefix = format!("users/{user_scope}/docs/");
    let images_prefix = format!("users/{user_scope}/images/");
    if object_key.starts_with(&docs_prefix) || object_key.starts_with(&images_prefix) {
        return Ok(());
    }

    Err(BizError::new(
        app_error::CONVERSATION_REQUEST_INVALID,
        "invalid conversation file object key".to_string(),
    ))
}

fn ensure_object_key_sha256_matches(object_key: &str, sha256: &str) -> BizResult<()> {
    let sha256 = normalize_sha256(sha256)?;
    let file_stem = object_key
        .rsplit('/')
        .next()
        .and_then(|file_name| file_name.split('.').next())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_stem == sha256 {
        return Ok(());
    }

    Err(BizError::new(
        app_error::CONVERSATION_REQUEST_INVALID,
        "object key does not match sha256".to_string(),
    ))
}

fn normalize_sha256(value: &str) -> BizResult<String> {
    let value = value
        .trim()
        .trim_end_matches(".sha256")
        .to_ascii_lowercase();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "invalid sha256".to_string(),
        ))
    }
}

fn file_service_user_scope_key(user_id: &Uuid) -> String {
    encode_base62_u128(user_id.as_u128())
}

fn encode_base62_u128(mut value: u128) -> String {
    const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    if value == 0 {
        return "0".to_string();
    }

    let mut output = Vec::new();
    while value > 0 {
        let index = (value % 62) as usize;
        output.push(ALPHABET[index] as char);
        value /= 62;
    }
    output.iter().rev().collect()
}

fn normalize_extraction_status(value: Option<String>) -> BizResult<String> {
    let value =
        normalize_optional(value).unwrap_or_else(|| EXTRACTION_STATUS_EXTRACTED.to_string());
    match value.as_str() {
        "ready" | "success" => Ok(EXTRACTION_STATUS_EXTRACTED.to_string()),
        EXTRACTION_STATUS_PENDING
        | EXTRACTION_STATUS_EXTRACTED
        | EXTRACTION_STATUS_FAILED
        | EXTRACTION_STATUS_UNSUPPORTED => Ok(value),
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("invalid extraction status: {value}"),
        )),
    }
}

#[derive(Debug, Clone)]
struct ServerExtractionPlan {
    job_id: String,
    status: String,
    content_kind: String,
    parser: String,
    text: Option<String>,
    failure_reason: Option<String>,
    message: String,
    server_can_read_object_body: bool,
    adapter: String,
    vision_ready: bool,
    extra_metadata: Value,
}

fn build_file_extraction_status_response(
    file: &ConversationFile,
) -> ConversationFileExtractionStatusResponse {
    let extraction = file.metadata.get("extraction");
    let status = file
        .metadata
        .get("extractionStatus")
        .and_then(Value::as_str)
        .or_else(|| {
            extraction
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str)
        })
        .unwrap_or(EXTRACTION_STATUS_PENDING)
        .to_string();
    let content_kind = extraction
        .and_then(|value| value.get("contentKind"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let parser = extraction
        .and_then(|value| value.get("parser"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let source = extraction
        .and_then(|value| value.get("source"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let language = extraction
        .and_then(|value| value.get("language"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let summary = file
        .metadata
        .get("extractedText")
        .and_then(Value::as_str)
        .or_else(|| {
            extraction
                .and_then(|value| value.get("text"))
                .and_then(Value::as_str)
        })
        .map(|value| compact_text(value, 500));
    let text_chars = extraction
        .and_then(|value| value.get("textChars"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .or_else(|| summary.as_deref().map(str::chars).map(Iterator::count))
        .unwrap_or(0);
    let failure_reason = extraction
        .and_then(|value| value.get("failureReason"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let vision_ready = extraction
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("visionReady"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let updated_at = extraction
        .and_then(|value| value.get("updatedAt"))
        .and_then(Value::as_str)
        .and_then(|value| OffsetDateTime::parse(value, &Rfc3339).ok())
        .unwrap_or(file.created_at);

    ConversationFileExtractionStatusResponse {
        file_id: uuid_to_string(file.id),
        status,
        content_kind,
        parser,
        source,
        language,
        summary,
        text_chars,
        readable: text_chars > 0,
        vision_ready,
        failure_reason,
        metadata: file.metadata.clone(),
        updated_at,
    }
}

fn classify_server_extraction_plan(file: &ConversationFile) -> ServerExtractionPlan {
    let (mime_type, file_name) = normalized_file_identity(file);
    let job_id = format!("server-extraction-{}", file.id);
    if is_browser_vision_image(&mime_type, &file_name) {
        return ServerExtractionPlan {
            job_id,
            status: EXTRACTION_STATUS_EXTRACTED.to_string(),
            content_kind: "image_vision_input".to_string(),
            parser: "server_status_classifier".to_string(),
            text: Some(
                "Image is available as a vision input when explicitly selected.".to_string(),
            ),
            failure_reason: None,
            message: "image can be used by the vision model when the frontend sends it as \
                      metadata.attachments.images"
                .to_string(),
            server_can_read_object_body: false,
            adapter: "status_classifier".to_string(),
            vision_ready: true,
            extra_metadata: json!({}),
        };
    }
    if is_server_pdf_extractable(&mime_type, &file_name) {
        return ServerExtractionPlan {
            job_id,
            status: EXTRACTION_STATUS_PENDING.to_string(),
            content_kind: "pdf".to_string(),
            parser: "server_pdf_text".to_string(),
            text: None,
            failure_reason: None,
            message: "PDF text-layer extraction requires object-body download".to_string(),
            server_can_read_object_body: false,
            adapter: "server_pdf_text".to_string(),
            vision_ready: false,
            extra_metadata: json!({}),
        };
    }
    if is_server_ooxml_extractable(&mime_type, &file_name) {
        return ServerExtractionPlan {
            job_id,
            status: EXTRACTION_STATUS_PENDING.to_string(),
            content_kind: inferred_content_kind(&mime_type, &file_name),
            parser: "server_ooxml_text".to_string(),
            text: None,
            failure_reason: None,
            message: "OOXML text extraction requires object-body download".to_string(),
            server_can_read_object_body: false,
            adapter: "server_ooxml_text".to_string(),
            vision_ready: false,
            extra_metadata: json!({}),
        };
    }
    if is_server_unsupported_binary(&mime_type, &file_name) {
        return ServerExtractionPlan {
            job_id,
            status: EXTRACTION_STATUS_UNSUPPORTED.to_string(),
            content_kind: inferred_content_kind(&mime_type, &file_name),
            parser: "server_status_classifier".to_string(),
            text: None,
            failure_reason: Some(
                "server object-body parser is not enabled for this file type yet".to_string(),
            ),
            message: "server parser is not enabled for this file type; file remains unavailable \
                      as readable text"
                .to_string(),
            server_can_read_object_body: false,
            adapter: "status_classifier".to_string(),
            vision_ready: false,
            extra_metadata: json!({}),
        };
    }
    ServerExtractionPlan {
        job_id,
        status: EXTRACTION_STATUS_PENDING.to_string(),
        content_kind: inferred_content_kind(&mime_type, &file_name),
        parser: "server_status_classifier".to_string(),
        text: None,
        failure_reason: None,
        message: "file is waiting for a text extraction adapter or trusted extraction upload"
            .to_string(),
        server_can_read_object_body: false,
        adapter: "status_classifier".to_string(),
        vision_ready: false,
        extra_metadata: json!({}),
    }
}

async fn build_server_text_extraction_plan(
    storage: &ObjectStorage,
    file: &ConversationFile,
) -> ServerExtractionPlan {
    if let Some(file_size) = file.file_size.filter(|value| *value > 0)
        && file_size as usize > MAX_SERVER_TEXT_OBJECT_BYTES
    {
        return build_server_text_size_guard_plan(file, file_size as usize, false);
    }

    match storage
        .download_object(DownloadObjectRequest {
            bucket: BucketKind::Private,
            key: file.object_key.clone(),
        })
        .await
    {
        Ok(bytes) if bytes.len() > MAX_SERVER_TEXT_OBJECT_BYTES => {
            build_server_text_size_guard_plan(file, bytes.len(), true)
        }
        Ok(bytes) => build_server_text_plan_from_bytes(file, bytes.to_vec()),
        Err(err) => build_server_text_failure_plan(
            file,
            "server_text",
            false,
            format!("server object download failed: {err}"),
        ),
    }
}

async fn build_server_pdf_text_extraction_plan(
    storage: &ObjectStorage,
    file: &ConversationFile,
) -> ServerExtractionPlan {
    if let Some(file_size) = file.file_size.filter(|value| *value > 0)
        && file_size as usize > MAX_SERVER_TEXT_OBJECT_BYTES
    {
        return build_server_pdf_size_guard_plan(file, file_size as usize, false);
    }

    match storage
        .download_object(DownloadObjectRequest {
            bucket: BucketKind::Private,
            key: file.object_key.clone(),
        })
        .await
    {
        Ok(bytes) if bytes.len() > MAX_SERVER_TEXT_OBJECT_BYTES => {
            build_server_pdf_size_guard_plan(file, bytes.len(), true)
        }
        Ok(bytes) => build_server_pdf_text_plan_from_bytes(file, bytes.to_vec()),
        Err(err) => build_server_pdf_failure_plan(
            file,
            false,
            format!("server PDF object download failed: {err}"),
        ),
    }
}

async fn build_server_ooxml_extraction_plan(
    storage: &ObjectStorage,
    file: &ConversationFile,
) -> ServerExtractionPlan {
    if let Some(file_size) = file.file_size.filter(|value| *value > 0)
        && file_size as usize > MAX_SERVER_TEXT_OBJECT_BYTES
    {
        return build_server_ooxml_size_guard_plan(file, file_size as usize, false);
    }

    match storage
        .download_object(DownloadObjectRequest {
            bucket: BucketKind::Private,
            key: file.object_key.clone(),
        })
        .await
    {
        Ok(bytes) if bytes.len() > MAX_SERVER_TEXT_OBJECT_BYTES => {
            build_server_ooxml_size_guard_plan(file, bytes.len(), true)
        }
        Ok(bytes) => build_server_ooxml_plan_from_bytes(file, bytes.to_vec()),
        Err(err) => build_server_ooxml_failure_plan(
            file,
            false,
            format!("server OOXML object download failed: {err}"),
        ),
    }
}

fn build_server_text_plan_from_bytes(
    file: &ConversationFile,
    bytes: Vec<u8>,
) -> ServerExtractionPlan {
    let object_bytes = bytes.len();
    let text = match String::from_utf8(bytes) {
        Ok(text) => text.trim().to_string(),
        Err(_) => {
            return build_server_text_failure_plan(
                file,
                "server_text",
                true,
                "server text extraction failed: object is not valid UTF-8".to_string(),
            );
        }
    };
    if text.is_empty() {
        return build_server_text_failure_plan(
            file,
            "server_text",
            true,
            "server text extraction failed: extracted text is empty".to_string(),
        );
    }

    let original_text_chars = text.chars().count();
    let truncated = original_text_chars > MAX_EXTRACTED_TEXT_CHARS;
    let extracted_text = if truncated {
        text.chars().take(MAX_EXTRACTED_TEXT_CHARS).collect()
    } else {
        text
    };
    let content_kind = inferred_content_kind_for_file(file);
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_EXTRACTED.to_string(),
        content_kind,
        parser: "server_text".to_string(),
        text: Some(extracted_text),
        failure_reason: None,
        message: if truncated {
            format!(
                "server extracted UTF-8 text and truncated it to {MAX_EXTRACTED_TEXT_CHARS} chars"
            )
        } else {
            "server extracted UTF-8 text from object storage".to_string()
        },
        server_can_read_object_body: true,
        adapter: "server_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "originalTextChars": original_text_chars,
            "truncated": truncated,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
        }),
    }
}

fn build_server_pdf_text_plan_from_bytes(
    file: &ConversationFile,
    bytes: Vec<u8>,
) -> ServerExtractionPlan {
    let object_bytes = bytes.len();
    if !bytes.starts_with(b"%PDF") {
        return build_server_pdf_failure_plan(
            file,
            true,
            "server PDF text extraction failed: object does not look like a PDF".to_string(),
        );
    }
    if contains_pdf_marker(&bytes, b"/Encrypt") {
        return build_server_pdf_failure_plan(
            file,
            true,
            "server PDF text extraction failed: encrypted PDFs are not supported".to_string(),
        );
    }

    let Some(text) = extract_pdf_text_layer(&bytes) else {
        return build_server_pdf_failure_plan(
            file,
            true,
            "server PDF text extraction found no readable text layer; OCR is not enabled"
                .to_string(),
        );
    };

    let original_text_chars = text.chars().count();
    let truncated = original_text_chars > MAX_EXTRACTED_TEXT_CHARS;
    let extracted_text = if truncated {
        text.chars().take(MAX_EXTRACTED_TEXT_CHARS).collect()
    } else {
        text
    };
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_EXTRACTED.to_string(),
        content_kind: "pdf".to_string(),
        parser: "server_pdf_text".to_string(),
        text: Some(extracted_text),
        failure_reason: None,
        message: if truncated {
            format!(
                "server extracted PDF text layer and truncated it to {MAX_EXTRACTED_TEXT_CHARS} \
                 chars"
            )
        } else {
            "server extracted readable PDF text layer from object storage".to_string()
        },
        server_can_read_object_body: true,
        adapter: "server_pdf_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "originalTextChars": original_text_chars,
            "truncated": truncated,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
            "ocrReady": false,
            "parserScope": "literal_pdf_text_strings_only",
        }),
    }
}

fn build_server_ooxml_plan_from_bytes(
    file: &ConversationFile,
    bytes: Vec<u8>,
) -> ServerExtractionPlan {
    let object_bytes = bytes.len();
    if !bytes.starts_with(b"PK\x03\x04") {
        return build_server_ooxml_failure_plan(
            file,
            true,
            "server OOXML text extraction failed: object does not look like a ZIP-based Office \
             file"
                .to_string(),
        );
    }
    let content_kind = inferred_content_kind_for_file(file);
    let zip = match read_ooxml_zip_entries(&bytes, &content_kind) {
        Ok(zip) => zip,
        Err(reason) => {
            return build_server_ooxml_pending_plan(
                file,
                object_bytes,
                true,
                format!("server OOXML text extraction is waiting for a full ZIP parser: {reason}"),
                0,
                0,
            );
        }
    };
    let text = if content_kind == "xlsx" {
        extract_xlsx_text_from_entries(&zip.entries)
    } else {
        extract_docx_text_from_entries(&zip.entries)
    };
    let Some(text) = text else {
        if zip.compressed_entry_count > 0 || zip.descriptor_entry_count > 0 {
            return build_server_ooxml_pending_plan(
                file,
                object_bytes,
                true,
                format!(
                    "server OOXML text extraction needs ZIP deflate/data-descriptor support for \
                     {} compressed and {} descriptor-based entr{}",
                    zip.compressed_entry_count,
                    zip.descriptor_entry_count,
                    if zip.compressed_entry_count + zip.descriptor_entry_count == 1 {
                        "y"
                    } else {
                        "ies"
                    }
                ),
                zip.compressed_entry_count,
                zip.descriptor_entry_count,
            );
        }
        return build_server_ooxml_failure_plan(
            file,
            true,
            "server OOXML text extraction failed: no readable document XML was found".to_string(),
        );
    };

    let original_text_chars = text.chars().count();
    let truncated = original_text_chars > MAX_EXTRACTED_TEXT_CHARS;
    let extracted_text = if truncated {
        text.chars().take(MAX_EXTRACTED_TEXT_CHARS).collect()
    } else {
        text
    };
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_EXTRACTED.to_string(),
        content_kind,
        parser: "server_ooxml_text".to_string(),
        text: Some(extracted_text),
        failure_reason: None,
        message: if truncated {
            format!(
                "server extracted OOXML text and truncated it to {MAX_EXTRACTED_TEXT_CHARS} chars"
            )
        } else {
            "server extracted readable OOXML XML text from object storage".to_string()
        },
        server_can_read_object_body: true,
        adapter: "server_ooxml_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "originalTextChars": original_text_chars,
            "truncated": truncated,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
            "parserScope": "stored_zip_xml_entries_only",
            "compressedEntryCount": zip.compressed_entry_count,
            "descriptorEntryCount": zip.descriptor_entry_count,
            "readableEntryCount": zip.entries.len(),
        }),
    }
}

fn build_server_pdf_size_guard_plan(
    file: &ConversationFile,
    object_bytes: usize,
    downloaded: bool,
) -> ServerExtractionPlan {
    let failure_reason = format!(
        "server PDF text extraction skipped: object size {object_bytes} bytes exceeds inline \
         limit {MAX_SERVER_TEXT_OBJECT_BYTES} bytes"
    );
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_PENDING.to_string(),
        content_kind: "pdf".to_string(),
        parser: "server_pdf_text_guard".to_string(),
        text: None,
        failure_reason: Some(failure_reason.clone()),
        message: "PDF is waiting for a chunked parser, OCR, or external extraction adapter"
            .to_string(),
        server_can_read_object_body: downloaded,
        adapter: "server_pdf_text_guard".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "downloaded": downloaded,
            "reason": failure_reason,
            "ocrReady": false,
        }),
    }
}

fn build_server_ooxml_size_guard_plan(
    file: &ConversationFile,
    object_bytes: usize,
    downloaded: bool,
) -> ServerExtractionPlan {
    let failure_reason = format!(
        "server OOXML text extraction skipped: object size {object_bytes} bytes exceeds inline \
         limit {MAX_SERVER_TEXT_OBJECT_BYTES} bytes"
    );
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_PENDING.to_string(),
        content_kind: inferred_content_kind_for_file(file),
        parser: "server_ooxml_text_guard".to_string(),
        text: None,
        failure_reason: Some(failure_reason.clone()),
        message: "Office file is waiting for a chunked OOXML parser or external extraction adapter"
            .to_string(),
        server_can_read_object_body: downloaded,
        adapter: "server_ooxml_text_guard".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "downloaded": downloaded,
            "reason": failure_reason,
            "parserScope": "stored_zip_xml_entries_only",
        }),
    }
}

fn build_server_pdf_failure_plan(
    file: &ConversationFile,
    server_can_read_object_body: bool,
    failure_reason: String,
) -> ServerExtractionPlan {
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_FAILED.to_string(),
        content_kind: "pdf".to_string(),
        parser: "server_pdf_text".to_string(),
        text: None,
        failure_reason: Some(compact_text(&failure_reason, 500)),
        message: "server PDF text extraction failed; PDF is unavailable as readable context"
            .to_string(),
        server_can_read_object_body,
        adapter: "server_pdf_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "failureReason": failure_reason,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
            "ocrReady": false,
        }),
    }
}

fn build_server_ooxml_pending_plan(
    file: &ConversationFile,
    object_bytes: usize,
    server_can_read_object_body: bool,
    failure_reason: String,
    compressed_entry_count: usize,
    descriptor_entry_count: usize,
) -> ServerExtractionPlan {
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_PENDING.to_string(),
        content_kind: inferred_content_kind_for_file(file),
        parser: "server_ooxml_text_guard".to_string(),
        text: None,
        failure_reason: Some(compact_text(&failure_reason, 500)),
        message: "Office file is waiting for ZIP deflate support or an external extraction adapter"
            .to_string(),
        server_can_read_object_body,
        adapter: "server_ooxml_text_guard".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "failureReason": failure_reason,
            "objectBytes": object_bytes,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
            "parserScope": "stored_zip_xml_entries_only",
            "compressedEntryCount": compressed_entry_count,
            "descriptorEntryCount": descriptor_entry_count,
        }),
    }
}

fn build_server_ooxml_failure_plan(
    file: &ConversationFile,
    server_can_read_object_body: bool,
    failure_reason: String,
) -> ServerExtractionPlan {
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_FAILED.to_string(),
        content_kind: inferred_content_kind_for_file(file),
        parser: "server_ooxml_text".to_string(),
        text: None,
        failure_reason: Some(compact_text(&failure_reason, 500)),
        message: "server OOXML text extraction failed; Office file is unavailable as readable \
                  context"
            .to_string(),
        server_can_read_object_body,
        adapter: "server_ooxml_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "failureReason": failure_reason,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
            "parserScope": "stored_zip_xml_entries_only",
        }),
    }
}

fn build_server_text_size_guard_plan(
    file: &ConversationFile,
    object_bytes: usize,
    downloaded: bool,
) -> ServerExtractionPlan {
    let failure_reason = format!(
        "server text extraction skipped: object size {object_bytes} bytes exceeds inline limit \
         {MAX_SERVER_TEXT_OBJECT_BYTES} bytes"
    );
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_PENDING.to_string(),
        content_kind: inferred_content_kind_for_file(file),
        parser: "server_text_guard".to_string(),
        text: None,
        failure_reason: Some(failure_reason.clone()),
        message: "file is waiting for a chunked parser, table profiler, or external extraction \
                  adapter"
            .to_string(),
        server_can_read_object_body: downloaded,
        adapter: "server_text_guard".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "objectBytes": object_bytes,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "downloaded": downloaded,
            "reason": failure_reason,
        }),
    }
}

fn build_server_text_failure_plan(
    file: &ConversationFile,
    parser: &str,
    server_can_read_object_body: bool,
    failure_reason: String,
) -> ServerExtractionPlan {
    ServerExtractionPlan {
        job_id: format!("server-extraction-{}", file.id),
        status: EXTRACTION_STATUS_FAILED.to_string(),
        content_kind: inferred_content_kind_for_file(file),
        parser: parser.to_string(),
        text: None,
        failure_reason: Some(compact_text(&failure_reason, 500)),
        message: "server text extraction failed; file is unavailable as readable context"
            .to_string(),
        server_can_read_object_body,
        adapter: "server_text".to_string(),
        vision_ready: false,
        extra_metadata: json!({
            "failureReason": failure_reason,
            "maxObjectBytes": MAX_SERVER_TEXT_OBJECT_BYTES,
            "maxTextChars": MAX_EXTRACTED_TEXT_CHARS,
        }),
    }
}

#[derive(Debug, Clone)]
struct OoxmlZipRead {
    entries: Vec<OoxmlZipEntry>,
    compressed_entry_count: usize,
    descriptor_entry_count: usize,
}

#[derive(Debug, Clone)]
struct OoxmlZipEntry {
    name: String,
    text: String,
}

fn read_ooxml_zip_entries(bytes: &[u8], content_kind: &str) -> Result<OoxmlZipRead, String> {
    let mut index = 0usize;
    let mut entries = Vec::new();
    let mut compressed_entry_count = 0usize;
    let mut descriptor_entry_count = 0usize;
    while index + 30 <= bytes.len() {
        if &bytes[index .. index + 4] != b"PK\x03\x04" {
            if &bytes[index .. index + 4] == b"PK\x01\x02"
                || &bytes[index .. index + 4] == b"PK\x05\x06"
            {
                break;
            }
            index += 1;
            continue;
        }
        let flags = read_le_u16(bytes, index + 6)?;
        let method = read_le_u16(bytes, index + 8)?;
        let compressed_size = read_le_u32(bytes, index + 18)? as usize;
        let uncompressed_size = read_le_u32(bytes, index + 22)? as usize;
        let file_name_len = read_le_u16(bytes, index + 26)? as usize;
        let extra_len = read_le_u16(bytes, index + 28)? as usize;
        let name_start = index + 30;
        let name_end = name_start
            .checked_add(file_name_len)
            .ok_or_else(|| "zip file name length overflow".to_string())?;
        let data_start = name_end
            .checked_add(extra_len)
            .ok_or_else(|| "zip extra length overflow".to_string())?;
        if data_start > bytes.len() {
            return Err("zip local header exceeds object bounds".to_string());
        }
        let name = String::from_utf8_lossy(&bytes[name_start .. name_end]).replace('\\', "/");
        if flags & 0x08 != 0 {
            descriptor_entry_count += 1;
            break;
        }
        if method == 8 {
            compressed_entry_count += 1;
            index = data_start
                .checked_add(compressed_size)
                .ok_or_else(|| "zip compressed entry size overflow".to_string())?;
            continue;
        }
        if method != 0 {
            index = data_start
                .checked_add(compressed_size)
                .ok_or_else(|| "zip entry size overflow".to_string())?;
            continue;
        }
        if compressed_size != uncompressed_size {
            return Err("stored zip entry has mismatched compressed/uncompressed size".to_string());
        }
        let data_end = data_start
            .checked_add(compressed_size)
            .ok_or_else(|| "zip data size overflow".to_string())?;
        if data_end > bytes.len() {
            return Err("zip entry data exceeds object bounds".to_string());
        }
        if is_relevant_ooxml_xml_entry(content_kind, &name) {
            let text = String::from_utf8_lossy(&bytes[data_start .. data_end]).to_string();
            entries.push(OoxmlZipEntry { name, text });
        }
        index = data_end;
    }
    Ok(OoxmlZipRead {
        entries,
        compressed_entry_count,
        descriptor_entry_count,
    })
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| "u16 offset overflow".to_string())?;
    let slice = bytes
        .get(offset .. end)
        .ok_or_else(|| "zip header is truncated".to_string())?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "u32 offset overflow".to_string())?;
    let slice = bytes
        .get(offset .. end)
        .ok_or_else(|| "zip header is truncated".to_string())?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn is_relevant_ooxml_xml_entry(content_kind: &str, name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    if content_kind == "docx" {
        name == "word/document.xml"
            || name.starts_with("word/header")
            || name.starts_with("word/footer")
            || name == "word/footnotes.xml"
            || name == "word/endnotes.xml"
    } else if content_kind == "xlsx" {
        name == "xl/sharedstrings.xml"
            || name.starts_with("xl/worksheets/sheet")
            || name == "xl/workbook.xml"
    } else {
        false
    }
}

fn extract_docx_text_from_entries(entries: &[OoxmlZipEntry]) -> Option<String> {
    let mut parts = Vec::new();
    for entry in entries {
        if entry.name.eq_ignore_ascii_case("word/document.xml")
            && let Some(text) = extract_xml_text_nodes(&entry.text)
        {
            parts.push(text);
        }
    }
    for entry in entries {
        if !entry.name.eq_ignore_ascii_case("word/document.xml")
            && let Some(text) = extract_xml_text_nodes(&entry.text)
        {
            parts.push(text);
        }
    }
    normalize_office_extracted_text(&parts.join("\n"))
}

fn extract_xlsx_text_from_entries(entries: &[OoxmlZipEntry]) -> Option<String> {
    let shared_strings = entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case("xl/sharedstrings.xml"))
        .and_then(|entry| extract_xlsx_shared_strings(&entry.text))
        .unwrap_or_default();
    let mut parts = Vec::new();
    if !shared_strings.is_empty() {
        parts.push(format!("Shared strings:\n{}", shared_strings.join("\n")));
    }
    let mut sheet_index = 1usize;
    for entry in entries.iter().filter(|entry| {
        entry
            .name
            .to_ascii_lowercase()
            .starts_with("xl/worksheets/sheet")
    }) {
        if let Some(sheet_text) = extract_xlsx_sheet_text(&entry.text, &shared_strings) {
            parts.push(format!("Sheet {sheet_index}:\n{sheet_text}"));
            sheet_index += 1;
        }
    }
    normalize_office_extracted_text(&parts.join("\n"))
}

fn extract_xlsx_shared_strings(xml: &str) -> Option<Vec<String>> {
    let mut strings = Vec::new();
    let mut search_from = 0usize;
    while let Some(start) = xml[search_from ..].find("<si") {
        let si_start = search_from + start;
        let Some(open_end) = xml[si_start ..].find('>') else {
            break;
        };
        let content_start = si_start + open_end + 1;
        let Some(end) = xml[content_start ..].find("</si>") else {
            break;
        };
        let content_end = content_start + end;
        if let Some(text) = extract_xml_text_nodes(&xml[content_start .. content_end]) {
            strings.push(text);
        }
        search_from = content_end + "</si>".len();
    }
    (!strings.is_empty()).then_some(strings)
}

fn extract_xlsx_sheet_text(xml: &str, shared_strings: &[String]) -> Option<String> {
    let mut rows = Vec::new();
    let mut search_from = 0usize;
    while let Some(start) = xml[search_from ..].find("<row") {
        let row_start = search_from + start;
        let Some(open_end) = xml[row_start ..].find('>') else {
            break;
        };
        let content_start = row_start + open_end + 1;
        let Some(end) = xml[content_start ..].find("</row>") else {
            break;
        };
        let content_end = content_start + end;
        let cells = extract_xlsx_cells(&xml[content_start .. content_end], shared_strings);
        if !cells.is_empty() {
            rows.push(cells.join("\t"));
        }
        search_from = content_end + "</row>".len();
    }
    if rows.is_empty() {
        extract_xml_text_nodes(xml)
    } else {
        Some(rows.join("\n"))
    }
}

fn extract_xlsx_cells(row_xml: &str, shared_strings: &[String]) -> Vec<String> {
    let mut cells = Vec::new();
    let mut search_from = 0usize;
    while let Some(start) = row_xml[search_from ..].find("<c") {
        let cell_start = search_from + start;
        let Some(open_end) = row_xml[cell_start ..].find('>') else {
            break;
        };
        let cell_header = &row_xml[cell_start .. cell_start + open_end + 1];
        let content_start = cell_start + open_end + 1;
        let Some(end) = row_xml[content_start ..].find("</c>") else {
            break;
        };
        let content_end = content_start + end;
        let cell_xml = &row_xml[content_start .. content_end];
        if let Some(value) = extract_xlsx_cell_value(cell_header, cell_xml, shared_strings) {
            cells.push(value);
        }
        search_from = content_end + "</c>".len();
    }
    cells
}

fn extract_xlsx_cell_value(
    cell_header: &str,
    cell_xml: &str,
    shared_strings: &[String],
) -> Option<String> {
    if let Some(inline) =
        extract_tag_text(cell_xml, "is").or_else(|| extract_tag_text(cell_xml, "t"))
    {
        return Some(inline);
    }
    let value = extract_tag_text_allow_short(cell_xml, "v")?;
    if cell_header.contains("t=\"s\"") || cell_header.contains("t='s'") {
        return value
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|index| shared_strings.get(index).cloned())
            .or(Some(value));
    }
    Some(value)
}

fn extract_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let open_end = xml[start ..].find('>')?;
    let content_start = start + open_end + 1;
    let end = xml[content_start ..].find(&close)?;
    extract_xml_text_nodes(&xml[content_start .. content_start + end])
}

fn extract_tag_text_allow_short(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let open_end = xml[start ..].find('>')?;
    let content_start = start + open_end + 1;
    let end = xml[content_start ..].find(&close)?;
    let text = strip_xml_tags_and_decode(&xml[content_start .. content_start + end]);
    (!text.trim().is_empty()).then_some(text)
}

fn extract_xml_text_nodes(xml: &str) -> Option<String> {
    normalize_office_extracted_text(&strip_xml_tags_and_decode(xml))
}

fn strip_xml_tags_and_decode(xml: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut entity = String::new();
    let mut in_entity = false;
    for ch in xml.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                output.push(' ');
            }
            continue;
        }
        if in_entity {
            if ch == ';' {
                output.push_str(&decode_xml_entity(&entity));
                entity.clear();
                in_entity = false;
            } else if entity.len() < 12 {
                entity.push(ch);
            } else {
                output.push('&');
                output.push_str(&entity);
                output.push(ch);
                entity.clear();
                in_entity = false;
            }
            continue;
        }
        match ch {
            '<' => in_tag = true,
            '&' => in_entity = true,
            _ => output.push(ch),
        }
    }
    if in_entity {
        output.push('&');
        output.push_str(&entity);
    }
    output
}

fn decode_xml_entity(entity: &str) -> String {
    match entity {
        "amp" => "&".to_string(),
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "quot" => "\"".to_string(),
        "apos" => "'".to_string(),
        value if value.starts_with("#x") => u32::from_str_radix(&value[2 ..], 16)
            .ok()
            .and_then(char::from_u32)
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| format!("&{entity};")),
        value if value.starts_with('#') => value[1 ..]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| format!("&{entity};")),
        _ => format!("&{entity};"),
    }
}

fn normalize_office_extracted_text(value: &str) -> Option<String> {
    let lines = value
        .lines()
        .map(normalize_office_line)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let text = if lines.is_empty() {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        lines.join("\n")
    };
    if text
        .chars()
        .filter(|ch| ch.is_alphanumeric() || is_cjk(*ch))
        .count()
        >= 2
    {
        Some(text)
    } else {
        None
    }
}

fn normalize_office_line(value: &str) -> String {
    value
        .split('\t')
        .map(|cell| cell.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\t")
}

fn contains_pdf_marker(bytes: &[u8], marker: &[u8]) -> bool {
    bytes
        .windows(marker.len())
        .any(|window| window.eq_ignore_ascii_case(marker))
}

fn extract_pdf_text_layer(bytes: &[u8]) -> Option<String> {
    let source = String::from_utf8_lossy(bytes);
    let mut fragments = Vec::new();
    let chars = source.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '(' {
            let (literal, next_index) = parse_pdf_literal_string(&chars, index + 1);
            index = next_index;
            if let Some(text) = literal.and_then(|value| normalize_pdf_text_fragment(&value)) {
                fragments.push(text);
            }
            continue;
        }
        if chars[index] == '<'
            && chars.get(index + 1) != Some(&'<')
            && let Some((hex, next_index)) = parse_pdf_hex_string(&chars, index + 1)
        {
            index = next_index;
            if let Some(text) =
                decode_pdf_hex_text(&hex).and_then(|value| normalize_pdf_text_fragment(&value))
            {
                fragments.push(text);
            }
            continue;
        }
        index += 1;
    }

    let joined = fragments.join(" ");
    normalize_pdf_extracted_text(&joined)
}

fn parse_pdf_literal_string(chars: &[char], mut index: usize) -> (Option<String>, usize) {
    let mut output = String::new();
    let mut depth = 1usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '\\' {
            let Some(next) = chars.get(index + 1).copied() else {
                return (Some(output), index + 1);
            };
            match next {
                'n' => output.push('\n'),
                'r' => output.push('\n'),
                't' => output.push('\t'),
                'b' | 'f' => output.push(' '),
                '(' | ')' | '\\' => output.push(next),
                '\r' | '\n' => {}
                '0' ..= '7' => {
                    let mut octal = String::new();
                    octal.push(next);
                    let mut consumed = 1usize;
                    while consumed < 3 {
                        let Some(candidate) = chars.get(index + 1 + consumed).copied() else {
                            break;
                        };
                        if !matches!(candidate, '0' ..= '7') {
                            break;
                        }
                        octal.push(candidate);
                        consumed += 1;
                    }
                    if let Ok(value) = u8::from_str_radix(&octal, 8) {
                        output.push(value as char);
                    }
                    index += consumed - 1;
                }
                _ => output.push(next),
            }
            index += 2;
            continue;
        }
        if ch == '(' {
            depth += 1;
            output.push(ch);
            index += 1;
            continue;
        }
        if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return (Some(output), index + 1);
            }
            output.push(ch);
            index += 1;
            continue;
        }
        output.push(ch);
        index += 1;
    }
    (None, index)
}

fn parse_pdf_hex_string(chars: &[char], mut index: usize) -> Option<(String, usize)> {
    let mut output = String::new();
    while index < chars.len() {
        let ch = chars[index];
        if ch == '>' {
            return Some((output, index + 1));
        }
        if ch.is_ascii_hexdigit() {
            output.push(ch);
        } else if !ch.is_whitespace() {
            return None;
        }
        index += 1;
    }
    None
}

fn decode_pdf_hex_text(hex: &str) -> Option<String> {
    if hex.len() < 2 {
        return None;
    }
    let mut normalized = hex.to_string();
    if !normalized.len().is_multiple_of(2) {
        normalized.push('0');
    }
    let mut bytes = Vec::new();
    let chars = normalized.as_bytes();
    for pair in chars.chunks(2) {
        let value = std::str::from_utf8(pair)
            .ok()
            .and_then(|value| u8::from_str_radix(value, 16).ok())?;
        bytes.push(value);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        let units = bytes[2 ..]
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2 {
                    Some(u16::from_be_bytes([pair[0], pair[1]]))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        return String::from_utf16(&units).ok();
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        let units = bytes[2 ..]
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2 {
                    Some(u16::from_le_bytes([pair[0], pair[1]]))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        return String::from_utf16(&units).ok();
    }
    if bytes.len() % 2 == 0 {
        let units = bytes
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2 {
                    Some(u16::from_be_bytes([pair[0], pair[1]]))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if let Ok(text) = String::from_utf16(&units)
            && text.chars().any(is_cjk)
        {
            return Some(text);
        }
    }
    Some(bytes.into_iter().map(char::from).collect())
}

fn normalize_pdf_text_fragment(value: &str) -> Option<String> {
    let normalized = value
        .chars()
        .map(|ch| {
            if ch.is_control() && ch != '\n' && ch != '\t' {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();
    let normalized = normalize_pdf_extracted_text(&normalized)?;
    let alpha_numeric_count = normalized
        .chars()
        .filter(|ch| ch.is_alphanumeric() || is_cjk(*ch))
        .count();
    if alpha_numeric_count < 2 {
        return None;
    }
    let operator_noise = [
        "obj",
        "endobj",
        "stream",
        "endstream",
        "xref",
        "trailer",
        "length",
        "filter",
        "font",
        "procset",
    ];
    let lower = normalized.to_ascii_lowercase();
    if operator_noise.iter().any(|noise| lower == *noise) {
        return None;
    }
    Some(normalized)
}

fn normalize_pdf_extracted_text(value: &str) -> Option<String> {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() >= 2 {
        Some(text)
    } else {
        None
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF
    )
}

fn build_server_extraction_extra_metadata(plan: &ServerExtractionPlan, mode: String) -> Value {
    let mut metadata = plan.extra_metadata.as_object().cloned().unwrap_or_default();
    metadata.insert("jobId".to_string(), Value::String(plan.job_id.clone()));
    metadata.insert("mode".to_string(), Value::String(mode));
    metadata.insert("reason".to_string(), Value::String(plan.message.clone()));
    metadata.insert(
        "serverCanReadObjectBody".to_string(),
        Value::Bool(plan.server_can_read_object_body),
    );
    metadata.insert("adapter".to_string(), Value::String(plan.adapter.clone()));
    metadata.insert("visionReady".to_string(), Value::Bool(plan.vision_ready));
    Value::Object(metadata)
}

fn normalized_file_identity(file: &ConversationFile) -> (String, String) {
    let mime_type = file
        .mime_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_name = file
        .file_name
        .as_deref()
        .or_else(|| file.object_key.rsplit('/').next())
        .unwrap_or_default()
        .to_ascii_lowercase();
    (mime_type, file_name)
}

fn is_server_text_extractable(mime_type: &str, file_name: &str) -> bool {
    let mime_type = mime_type
        .split(';')
        .next()
        .unwrap_or(mime_type)
        .trim()
        .to_ascii_lowercase();
    mime_type.starts_with("text/")
        || matches!(
            mime_type.as_str(),
            "application/json" | "application/ld+json" | "application/markdown"
        )
        || file_name.ends_with(".txt")
        || file_name.ends_with(".md")
        || file_name.ends_with(".markdown")
        || file_name.ends_with(".json")
        || file_name.ends_with(".jsonl")
        || file_name.ends_with(".csv")
        || file_name.ends_with(".tsv")
        || file_name.ends_with(".yaml")
        || file_name.ends_with(".yml")
        || file_name.ends_with(".toml")
        || file_name.ends_with(".xml")
        || file_name.ends_with(".html")
        || file_name.ends_with(".htm")
        || file_name.ends_with(".log")
        || file_name.ends_with(".sql")
        || file_name.ends_with(".rs")
        || file_name.ends_with(".go")
        || file_name.ends_with(".py")
        || file_name.ends_with(".js")
        || file_name.ends_with(".jsx")
        || file_name.ends_with(".ts")
        || file_name.ends_with(".tsx")
        || file_name.ends_with(".css")
        || file_name.ends_with(".scss")
        || file_name.ends_with(".sh")
        || file_name.ends_with(".ps1")
}

fn is_server_pdf_extractable(mime_type: &str, file_name: &str) -> bool {
    mime_type.contains("pdf") || file_name.ends_with(".pdf")
}

fn is_server_ooxml_extractable(mime_type: &str, file_name: &str) -> bool {
    mime_type.contains("officedocument.wordprocessingml.document")
        || mime_type.contains("officedocument.spreadsheetml.sheet")
        || file_name.ends_with(".docx")
        || file_name.ends_with(".xlsx")
}

fn inferred_content_kind_for_file(file: &ConversationFile) -> String {
    let (mime_type, file_name) = normalized_file_identity(file);
    inferred_content_kind(&mime_type, &file_name)
}

fn is_browser_vision_image(mime_type: &str, file_name: &str) -> bool {
    mime_type == "image/png"
        || mime_type == "image/jpeg"
        || mime_type == "image/webp"
        || file_name.ends_with(".png")
        || file_name.ends_with(".jpg")
        || file_name.ends_with(".jpeg")
        || file_name.ends_with(".webp")
}

fn is_server_unsupported_binary(mime_type: &str, file_name: &str) -> bool {
    mime_type.contains("word")
        || mime_type.contains("excel")
        || mime_type.contains("spreadsheet")
        || file_name.ends_with(".doc")
        || file_name.ends_with(".docx")
        || file_name.ends_with(".xls")
        || file_name.ends_with(".xlsx")
}

fn inferred_content_kind(mime_type: &str, file_name: &str) -> String {
    if mime_type.contains("pdf") || file_name.ends_with(".pdf") {
        "pdf".to_string()
    } else if mime_type.contains("word")
        || file_name.ends_with(".doc")
        || file_name.ends_with(".docx")
    {
        "docx".to_string()
    } else if mime_type.contains("excel")
        || mime_type.contains("spreadsheet")
        || file_name.ends_with(".xls")
        || file_name.ends_with(".xlsx")
    {
        "xlsx".to_string()
    } else if mime_type.contains("csv") || file_name.ends_with(".csv") {
        "csv".to_string()
    } else if mime_type.contains("tab-separated") || file_name.ends_with(".tsv") {
        "tsv".to_string()
    } else if mime_type.contains("markdown")
        || file_name.ends_with(".md")
        || file_name.ends_with(".markdown")
    {
        "markdown".to_string()
    } else if mime_type.contains("json")
        || file_name.ends_with(".json")
        || file_name.ends_with(".jsonl")
    {
        "json".to_string()
    } else if mime_type.contains("yaml")
        || file_name.ends_with(".yaml")
        || file_name.ends_with(".yml")
    {
        "yaml".to_string()
    } else if mime_type.contains("toml") || file_name.ends_with(".toml") {
        "toml".to_string()
    } else if mime_type.contains("xml") || file_name.ends_with(".xml") {
        "xml".to_string()
    } else if mime_type.contains("html")
        || file_name.ends_with(".html")
        || file_name.ends_with(".htm")
    {
        "html".to_string()
    } else if file_name.ends_with(".log") {
        "log".to_string()
    } else if is_code_text_file(file_name) {
        "code_text".to_string()
    } else if mime_type.starts_with("text/") || file_name.ends_with(".txt") {
        "plain_text".to_string()
    } else {
        "unknown".to_string()
    }
}

fn is_code_text_file(file_name: &str) -> bool {
    file_name.ends_with(".sql")
        || file_name.ends_with(".rs")
        || file_name.ends_with(".go")
        || file_name.ends_with(".py")
        || file_name.ends_with(".js")
        || file_name.ends_with(".jsx")
        || file_name.ends_with(".ts")
        || file_name.ends_with(".tsx")
        || file_name.ends_with(".css")
        || file_name.ends_with(".scss")
        || file_name.ends_with(".sh")
        || file_name.ends_with(".ps1")
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut output = value.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn build_initial_file_extraction_metadata() -> Value {
    json!({
        "extractionStatus": EXTRACTION_STATUS_PENDING,
        "extraction": {
            "status": EXTRACTION_STATUS_PENDING,
            "source": "upload",
            "parser": "unassigned",
            "updatedAt": now_rfc3339(),
            "metadata": {}
        }
    })
}

#[derive(Debug, Clone)]
struct FileExtractionMetadataPatch {
    status: String,
    text: Option<String>,
    content_kind: String,
    source: String,
    parser: String,
    language: Option<String>,
    failure_reason: Option<String>,
    text_chars: usize,
    extra_metadata: Value,
}

fn build_file_extraction_metadata(current: Value, patch: FileExtractionMetadataPatch) -> Value {
    let mut object = current.as_object().cloned().unwrap_or_default();
    object.insert(
        "extractionStatus".to_string(),
        Value::String(patch.status.clone()),
    );
    if let Some(text) = patch.text.as_ref() {
        object.insert("extractedText".to_string(), Value::String(text.clone()));
        object.insert("textContent".to_string(), Value::String(text.clone()));
    } else {
        object.remove("extractedText");
        object.remove("textContent");
    }

    let mut extraction = match object.remove("extraction") {
        Some(Value::Object(existing)) => existing,
        _ => Map::new(),
    };
    extraction.insert("status".to_string(), Value::String(patch.status));
    extraction.insert("contentKind".to_string(), Value::String(patch.content_kind));
    extraction.insert("source".to_string(), Value::String(patch.source));
    extraction.insert("parser".to_string(), Value::String(patch.parser));
    if let Some(text) = patch.text {
        extraction.insert("text".to_string(), Value::String(text));
    } else {
        extraction.remove("text");
    }
    extraction.insert(
        "textChars".to_string(),
        Value::Number(serde_json::Number::from(patch.text_chars)),
    );
    extraction.insert("updatedAt".to_string(), Value::String(now_rfc3339()));
    match patch.language {
        Some(language) => {
            extraction.insert("language".to_string(), Value::String(language));
        }
        None => {
            extraction.remove("language");
        }
    }
    match patch.failure_reason {
        Some(failure_reason) => {
            extraction.insert("failureReason".to_string(), Value::String(failure_reason));
        }
        None => {
            extraction.remove("failureReason");
        }
    }
    extraction.insert("metadata".to_string(), patch.extra_metadata);
    object.insert("extraction".to_string(), Value::Object(extraction));

    Value::Object(object)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

async fn commit_or_rollback<T>(tx: DbContext, result: BizResult<T>) -> BizResult<T> {
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conversation_file(
        file_name: &str,
        mime_type: &str,
        metadata: Value,
    ) -> ConversationFile {
        ConversationFile {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            message_id: None,
            bucket: "private".to_string(),
            object_key: format!("chat/user/conversation/{file_name}"),
            file_name: Some(file_name.to_string()),
            mime_type: Some(mime_type.to_string()),
            file_size: Some(128),
            sha256: None,
            visibility: "private".to_string(),
            storage_provider: "s3".to_string(),
            status: "active".to_string(),
            metadata,
            created_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        }
    }

    #[test]
    fn conversation_file_extraction_metadata_preserves_existing_fields() {
        let metadata = build_file_extraction_metadata(
            json!({
                "objectKey": "chat/u/c/file.pdf",
                "extraction": {
                    "status": "pending",
                    "source": "old-parser"
                }
            }),
            FileExtractionMetadataPatch {
                status: EXTRACTION_STATUS_EXTRACTED.to_string(),
                text: Some("商品卖点：防晒、轻薄".to_string()),
                content_kind: "markdown".to_string(),
                source: "manual".to_string(),
                parser: "manual".to_string(),
                language: Some("zh-CN".to_string()),
                failure_reason: None,
                text_chars: 10,
                extra_metadata: json!({"pageCount": 3}),
            },
        );

        assert_eq!(metadata["objectKey"], "chat/u/c/file.pdf");
        assert_eq!(metadata["extractionStatus"], EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(metadata["extractedText"], "商品卖点：防晒、轻薄");
        assert_eq!(metadata["textContent"], "商品卖点：防晒、轻薄");
        assert_eq!(
            metadata["extraction"]["status"],
            EXTRACTION_STATUS_EXTRACTED
        );
        assert_eq!(metadata["extraction"]["contentKind"], "markdown");
        assert_eq!(metadata["extraction"]["source"], "manual");
        assert_eq!(metadata["extraction"]["parser"], "manual");
        assert_eq!(metadata["extraction"]["language"], "zh-CN");
        assert_eq!(metadata["extraction"]["metadata"]["pageCount"], 3);
    }

    #[test]
    fn conversation_file_extraction_failure_clears_readable_text() {
        let metadata = build_file_extraction_metadata(
            json!({
                "extractedText": "旧正文",
                "textContent": "旧正文",
                "extraction": {
                    "status": "extracted",
                    "text": "旧正文"
                }
            }),
            FileExtractionMetadataPatch {
                status: EXTRACTION_STATUS_FAILED.to_string(),
                text: None,
                content_kind: "markdown".to_string(),
                source: "adapter".to_string(),
                parser: "pdf".to_string(),
                language: None,
                failure_reason: Some("encrypted pdf".to_string()),
                text_chars: 0,
                extra_metadata: json!({"attempt": 1}),
            },
        );

        assert_eq!(metadata["extractionStatus"], EXTRACTION_STATUS_FAILED);
        assert!(metadata.get("extractedText").is_none());
        assert!(metadata.get("textContent").is_none());
        assert_eq!(metadata["extraction"]["status"], EXTRACTION_STATUS_FAILED);
        assert_eq!(metadata["extraction"]["failureReason"], "encrypted pdf");
        assert!(metadata["extraction"].get("text").is_none());
    }

    #[test]
    fn conversation_file_extraction_status_normalizes_legacy_ready() {
        assert_eq!(
            normalize_extraction_status(Some("ready".to_string())).unwrap(),
            EXTRACTION_STATUS_EXTRACTED
        );
        assert_eq!(
            normalize_extraction_status(None).unwrap(),
            EXTRACTION_STATUS_EXTRACTED
        );
    }

    #[test]
    fn conversation_file_extraction_text_must_not_be_empty() {
        let err = normalize_required_text("   ".to_string(), "text").unwrap_err();
        assert_eq!(err.code(), app_error::CONVERSATION_REQUEST_INVALID);
    }

    #[test]
    fn conversation_file_extraction_status_reads_metadata() {
        let metadata = build_file_extraction_metadata(
            json!({}),
            FileExtractionMetadataPatch {
                status: EXTRACTION_STATUS_EXTRACTED.to_string(),
                text: Some("这是一段从附件中抽取的有效正文".to_string()),
                content_kind: "plain_text".to_string(),
                source: "trusted_api".to_string(),
                parser: "external_adapter".to_string(),
                language: Some("zh-CN".to_string()),
                failure_reason: None,
                text_chars: 15,
                extra_metadata: json!({"visionReady": false}),
            },
        );
        let file = test_conversation_file("notes.txt", "text/plain", metadata);

        let status = build_file_extraction_status_response(&file);

        assert_eq!(status.file_id, file.id.to_string());
        assert_eq!(status.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(status.content_kind.as_deref(), Some("plain_text"));
        assert_eq!(status.parser.as_deref(), Some("external_adapter"));
        assert_eq!(status.source.as_deref(), Some("trusted_api"));
        assert_eq!(status.language.as_deref(), Some("zh-CN"));
        assert_eq!(status.text_chars, 15);
        assert!(status.readable);
        assert!(!status.vision_ready);
        assert!(status.failure_reason.is_none());
        assert!(
            status
                .summary
                .as_deref()
                .unwrap_or_default()
                .contains("有效正文")
        );
    }

    #[test]
    fn conversation_file_extraction_status_reports_vision_ready_image() {
        let metadata = build_file_extraction_metadata(
            json!({}),
            FileExtractionMetadataPatch {
                status: EXTRACTION_STATUS_EXTRACTED.to_string(),
                text: Some("Image is available as a vision input.".to_string()),
                content_kind: "image_vision_input".to_string(),
                source: "server_job".to_string(),
                parser: "server_status_classifier".to_string(),
                language: None,
                failure_reason: None,
                text_chars: 37,
                extra_metadata: json!({"visionReady": true}),
            },
        );
        let file = test_conversation_file("hero.png", "image/png", metadata);

        let status = build_file_extraction_status_response(&file);

        assert_eq!(status.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(status.content_kind.as_deref(), Some("image_vision_input"));
        assert!(status.readable);
        assert!(status.vision_ready);
    }

    #[test]
    fn conversation_file_extraction_plan_classifies_browser_vision_images() {
        let file = test_conversation_file("hero.webp", "image/webp", json!({}));

        let plan = classify_server_extraction_plan(&file);

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(plan.content_kind, "image_vision_input");
        assert_eq!(plan.parser, "server_status_classifier");
        assert!(
            plan.text
                .as_deref()
                .unwrap_or_default()
                .contains("vision input")
        );
        assert!(plan.failure_reason.is_none());
    }

    #[test]
    fn conversation_file_extraction_plan_classifies_ooxml_as_pending_parser_candidate() {
        for (file_name, mime_type, expected_kind) in [
            (
                "brief.docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "docx",
            ),
            (
                "sales.xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "xlsx",
            ),
        ] {
            let file = test_conversation_file(file_name, mime_type, json!({}));

            let plan = classify_server_extraction_plan(&file);

            assert_eq!(plan.status, EXTRACTION_STATUS_PENDING);
            assert_eq!(plan.content_kind, expected_kind);
            assert_eq!(plan.parser, "server_ooxml_text");
            assert!(plan.text.is_none());
            assert!(plan.failure_reason.is_none());
        }
    }

    #[test]
    fn conversation_file_extraction_plan_classifies_pdf_as_pending_parser_candidate() {
        let file = test_conversation_file("brief.pdf", "application/pdf", json!({}));

        let plan = classify_server_extraction_plan(&file);

        assert_eq!(plan.status, EXTRACTION_STATUS_PENDING);
        assert_eq!(plan.content_kind, "pdf");
        assert_eq!(plan.parser, "server_pdf_text");
        assert!(plan.text.is_none());
        assert!(plan.failure_reason.is_none());
    }

    #[test]
    fn conversation_file_extraction_plan_keeps_csv_and_unknown_pending() {
        let file = test_conversation_file("archive.bin", "application/octet-stream", json!({}));

        let plan = classify_server_extraction_plan(&file);

        assert_eq!(plan.status, EXTRACTION_STATUS_PENDING);
        assert_eq!(plan.content_kind, "unknown");
        assert!(plan.text.is_none());
        assert!(plan.failure_reason.is_none());
    }

    #[test]
    fn conversation_file_extraction_recognizes_server_text_formats() {
        for (file_name, mime_type, expected_kind) in [
            ("notes.txt", "text/plain", "plain_text"),
            ("brief.md", "text/markdown", "markdown"),
            ("brief.markdown", "application/markdown", "markdown"),
            ("data.json", "application/json", "json"),
            ("events.jsonl", "application/x-ndjson", "json"),
            ("sales.csv", "text/csv", "csv"),
            ("sales.tsv", "text/tab-separated-values", "tsv"),
            ("config.yaml", "application/x-yaml", "yaml"),
            ("Cargo.toml", "application/octet-stream", "toml"),
            ("feed.xml", "application/xml", "xml"),
            ("page.html", "text/html", "html"),
            ("server.log", "application/octet-stream", "log"),
            ("script.ts", "application/octet-stream", "code_text"),
        ] {
            let file = test_conversation_file(file_name, mime_type, json!({}));
            let (normalized_mime_type, normalized_file_name) = normalized_file_identity(&file);

            assert!(is_server_text_extractable(
                &normalized_mime_type,
                &normalized_file_name
            ));
            assert_eq!(inferred_content_kind_for_file(&file), expected_kind);
        }
    }

    #[test]
    fn conversation_file_extraction_builds_server_text_plan() {
        let file = test_conversation_file("notes.txt", "text/plain", json!({}));

        let plan = build_server_text_plan_from_bytes(&file, b"  hello context  ".to_vec());

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(plan.content_kind, "plain_text");
        assert_eq!(plan.parser, "server_text");
        assert_eq!(plan.text.as_deref(), Some("hello context"));
        assert!(plan.failure_reason.is_none());
        assert!(plan.server_can_read_object_body);
        assert_eq!(plan.extra_metadata["objectBytes"], 17);
        assert_eq!(plan.extra_metadata["originalTextChars"], 13);
        assert_eq!(plan.extra_metadata["truncated"], false);
    }

    #[test]
    fn conversation_file_extraction_rejects_invalid_or_empty_server_text() {
        let file = test_conversation_file("notes.txt", "text/plain", json!({}));

        for bytes in [vec![0xff, 0xfe, 0xfd], b"   \n\t ".to_vec()] {
            let plan = build_server_text_plan_from_bytes(&file, bytes);

            assert_eq!(plan.status, EXTRACTION_STATUS_FAILED);
            assert_eq!(plan.content_kind, "plain_text");
            assert!(plan.text.is_none());
            assert!(plan.failure_reason.is_some());
        }
    }

    #[test]
    fn conversation_file_extraction_truncates_large_server_text() {
        let file = test_conversation_file("notes.txt", "text/plain", json!({}));
        let text = "字".repeat(MAX_EXTRACTED_TEXT_CHARS + 7);

        let plan = build_server_text_plan_from_bytes(&file, text.into_bytes());

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(
            plan.text.as_deref().unwrap().chars().count(),
            MAX_EXTRACTED_TEXT_CHARS
        );
        assert_eq!(
            plan.extra_metadata["originalTextChars"],
            MAX_EXTRACTED_TEXT_CHARS + 7
        );
        assert_eq!(plan.extra_metadata["truncated"], true);
    }

    #[test]
    fn conversation_file_extraction_builds_server_pdf_text_plan() {
        let file = test_conversation_file("brief.pdf", "application/pdf", json!({}));
        let bytes = br#"%PDF-1.4
1 0 obj
<< /Type /Catalog >>
endobj
2 0 obj
<< /Length 44 >>
stream
BT
(Hello PDF context) Tj
<4E2D6587> Tj
ET
endstream
endobj
%%EOF"#
            .to_vec();

        let plan = build_server_pdf_text_plan_from_bytes(&file, bytes);

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(plan.content_kind, "pdf");
        assert_eq!(plan.parser, "server_pdf_text");
        assert!(
            plan.text
                .as_deref()
                .is_some_and(|text| text.contains("Hello PDF context"))
        );
        assert!(
            plan.text
                .as_deref()
                .is_some_and(|text| text.contains("中文"))
        );
        assert!(plan.failure_reason.is_none());
        assert!(plan.server_can_read_object_body);
        assert_eq!(
            plan.extra_metadata["parserScope"],
            "literal_pdf_text_strings_only"
        );
        assert_eq!(plan.extra_metadata["ocrReady"], false);
    }

    #[test]
    fn conversation_file_extraction_rejects_encrypted_or_scanned_pdf() {
        let file = test_conversation_file("brief.pdf", "application/pdf", json!({}));

        for bytes in [
            b"%PDF-1.4\n<< /Encrypt 5 0 R >>\n%%EOF".to_vec(),
            b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n%%EOF".to_vec(),
            b"not a pdf".to_vec(),
        ] {
            let plan = build_server_pdf_text_plan_from_bytes(&file, bytes);

            assert_eq!(plan.status, EXTRACTION_STATUS_FAILED);
            assert_eq!(plan.content_kind, "pdf");
            assert_eq!(plan.parser, "server_pdf_text");
            assert!(plan.text.is_none());
            assert!(plan.failure_reason.is_some());
            assert_eq!(plan.extra_metadata["ocrReady"], false);
        }
    }

    #[test]
    fn conversation_file_extraction_builds_stored_docx_text_plan() {
        let file = test_conversation_file(
            "brief.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            json!({}),
        );
        let bytes = build_stored_zip_bytes(&[(
            "word/document.xml",
            r#"<w:document><w:body><w:p><w:r><w:t>商品卖点</w:t></w:r><w:r><w:t>轻薄&amp;防晒</w:t></w:r></w:p></w:body></w:document>"#,
        )]);

        let plan = build_server_ooxml_plan_from_bytes(&file, bytes);

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(plan.content_kind, "docx");
        assert_eq!(plan.parser, "server_ooxml_text");
        assert!(
            plan.text
                .as_deref()
                .is_some_and(|text| text.contains("商品卖点") && text.contains("轻薄&防晒"))
        );
        assert_eq!(
            plan.extra_metadata["parserScope"],
            "stored_zip_xml_entries_only"
        );
        assert_eq!(plan.extra_metadata["readableEntryCount"], 1);
    }

    #[test]
    fn conversation_file_extraction_builds_stored_xlsx_text_plan() {
        let file = test_conversation_file(
            "sales.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            json!({}),
        );
        let bytes = build_stored_zip_bytes(&[
            (
                "xl/sharedStrings.xml",
                r#"<sst><si><t>商品</t></si><si><t>销售额</t></si><si><t>咖啡</t></si></sst>"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                r#"<worksheet><sheetData><row><c t="s"><v>0</v></c><c t="s"><v>1</v></c></row><row><c t="s"><v>2</v></c><c><v>1288</v></c></row></sheetData></worksheet>"#,
            ),
        ]);

        let plan = build_server_ooxml_plan_from_bytes(&file, bytes);

        assert_eq!(plan.status, EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(plan.content_kind, "xlsx");
        assert_eq!(plan.parser, "server_ooxml_text");
        assert!(
            plan.text
                .as_deref()
                .is_some_and(|text| text.contains("商品\t销售额") && text.contains("咖啡\t1288"))
        );
        assert_eq!(
            plan.extra_metadata["parserScope"],
            "stored_zip_xml_entries_only"
        );
        assert_eq!(plan.extra_metadata["readableEntryCount"], 2);
    }

    #[test]
    fn conversation_file_extraction_keeps_deflated_ooxml_pending() {
        let file = test_conversation_file(
            "brief.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            json!({}),
        );
        let bytes = build_zip_local_header_bytes("word/document.xml", 8, b"compressed");

        let plan = build_server_ooxml_plan_from_bytes(&file, bytes);

        assert_eq!(plan.status, EXTRACTION_STATUS_PENDING);
        assert_eq!(plan.content_kind, "docx");
        assert_eq!(plan.parser, "server_ooxml_text_guard");
        assert!(plan.text.is_none());
        assert!(
            plan.failure_reason
                .as_deref()
                .unwrap_or_default()
                .contains("ZIP deflate")
        );
        assert_eq!(plan.extra_metadata["compressedEntryCount"], 1);
    }

    fn build_stored_zip_bytes(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (name, data) in entries {
            bytes.extend(build_zip_local_header_bytes(name, 0, data.as_bytes()));
        }
        bytes.extend(b"PK\x05\x06");
        bytes.extend([0u8; 18]);
        bytes
    }

    fn build_zip_local_header_bytes(name: &str, method: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(b"PK\x03\x04");
        bytes.extend(20u16.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(method.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend((name.len() as u16).to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(name.as_bytes());
        bytes.extend(data);
        bytes
    }
}
