use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use file_ingest::ViewOptions;
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
    api::{
        document_text_extraction_manager::DocumentTextExtractionManager,
        storage_object_analysis::{
            CompleteStorageObjectAnalysisRequest, FailStorageObjectAnalysisRequest,
            StorageObjectAnalysisApi,
        },
        storage_reference::{RetainStorageReferenceRequest, StorageReferenceApi},
    },
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
const MAX_EXTRACTED_TEXT_CHARS: usize = 100_000;
const EXTRACTION_STATUS_PENDING: &str = "pending";
const EXTRACTION_STATUS_EXTRACTED: &str = "extracted";
const EXTRACTION_STATUS_FAILED: &str = "failed";
const EXTRACTION_STATUS_UNSUPPORTED: &str = "unsupported";
const FILE_INGEST_PARSER: &str =
    crate::api::document_text_extraction_manager::DOCUMENT_TEXT_EXTRACTION_PROMPT_KEY;
const FILE_INGEST_PARSER_VERSION: &str =
    crate::api::document_text_extraction_manager::DOCUMENT_TEXT_EXTRACTION_PROMPT_VERSION;

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
                            strategy_id: None,
                            response_mode: None,
                            routing_mode: None,
                            collaboration_mode: None,
                            learning_level: None,
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
        let strategy_id = normalize_strategy_enum(
            "strategy_id",
            req.strategy_id
                .or_else(|| read_strategy_string(&content_json, "strategyId")),
            &["learning", "quick", "execution"],
        )?;
        let response_mode = normalize_strategy_enum(
            "response_mode",
            req.response_mode
                .or_else(|| read_strategy_string(&content_json, "responseMode")),
            &["learning", "quick", "execution"],
        )?;
        let routing_mode = normalize_strategy_enum(
            "routing_mode",
            req.routing_mode
                .or_else(|| read_strategy_string(&content_json, "routingMode")),
            &["commander", "fast"],
        )?;
        let collaboration_mode = normalize_strategy_enum(
            "collaboration_mode",
            req.collaboration_mode
                .or_else(|| read_strategy_string(&content_json, "collaborationMode")),
            &["auto", "single", "manual"],
        )?;
        let learning_level = normalize_strategy_enum(
            "learning_level",
            req.learning_level
                .or_else(|| read_strategy_string(&content_json, "learningLevel")),
            &["beginner", "higher_vocational", "professional"],
        )?;
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
                strategy_id,
                response_mode,
                routing_mode,
                collaboration_mode,
                learning_level,
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

    pub async fn extract_file_text(
        &self,
        storage: &ObjectStorage,
        user_id: String,
        file_id: String,
    ) -> BizResult<ConversationFileResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;
        tracing::debug!(
            file_id = %file.id,
            conversation_id = %file.conversation_id,
            object_key = %file.object_key,
            file_name = ?file.file_name,
            mime_type = ?file.mime_type,
            file_size = ?file.file_size,
            sha256 = ?file.sha256,
            "document text extraction requested"
        );

        let manager = DocumentTextExtractionManager::new(self.db.clone());
        let Some(task) = manager.ensure_task_for_conversation_file(file.id).await? else {
            tracing::debug!(
                file_id = %file.id,
                object_key = %file.object_key,
                file_name = ?file.file_name,
                mime_type = ?file.mime_type,
                "document text extraction skipped: unsupported document type"
            );
            return self
                .update_file_extraction(UpdateConversationFileExtractionRequest {
                    user_id: user_id.to_string(),
                    file_id: file_id.to_string(),
                    status: Some(EXTRACTION_STATUS_UNSUPPORTED.to_string()),
                    text: None,
                    content_kind: Some("unsupported".to_string()),
                    source: Some("server_file_ingest".to_string()),
                    parser: Some(FILE_INGEST_PARSER.to_string()),
                    language: None,
                    failure_reason: Some("unsupported document type".to_string()),
                    metadata: Some(json!({
                        "parserVersion": FILE_INGEST_PARSER_VERSION,
                    })),
                })
                .await;
        };

        let analysis_api = StorageObjectAnalysisApi::new(self.db.clone());
        if !StorageObjectAnalysisApi::can_process(&task.analysis.status) {
            tracing::debug!(
                file_id = %file.id,
                analysis_id = %task.analysis.id,
                analysis_status = %task.analysis.status,
                analysis_kind = %task.analysis.analysis_kind,
                storage_object_sha256 = %task.object.sha256,
                "document text extraction reused existing analysis"
            );
            return self
                .update_file_extraction(UpdateConversationFileExtractionRequest {
                    user_id: user_id.to_string(),
                    file_id: file_id.to_string(),
                    status: Some(EXTRACTION_STATUS_EXTRACTED.to_string()),
                    text: Some(format!(
                        "Document text extraction is already available in analysis {}.",
                        task.analysis.id
                    )),
                    content_kind: Some("document_text_analysis_ref".to_string()),
                    source: Some("server_file_ingest".to_string()),
                    parser: Some(FILE_INGEST_PARSER.to_string()),
                    language: None,
                    failure_reason: None,
                    metadata: Some(json!({
                        "parserVersion": FILE_INGEST_PARSER_VERSION,
                        "analysisId": task.analysis.id,
                        "analysisKind": task.analysis.analysis_kind,
                        "storageObjectSha256": task.object.sha256,
                    })),
                })
                .await;
        }

        tracing::debug!(
            file_id = %file.id,
            analysis_id = %task.analysis.id,
            analysis_kind = %task.analysis.analysis_kind,
            storage_object_sha256 = %task.object.sha256,
            object_key = %file.object_key,
            "document text extraction started"
        );
        analysis_api.mark_processing(task.analysis.id, None).await?;

        let bytes = match storage
            .download_object(DownloadObjectRequest {
                bucket: BucketKind::Private,
                key: file.object_key.clone(),
            })
            .await
        {
            Ok(bytes) => bytes,
            Err(err) => {
                let error_message = format!("download failed: {err}");
                tracing::warn!(
                    file_id = %file.id,
                    analysis_id = %task.analysis.id,
                    object_key = %file.object_key,
                    error = %err,
                    "document text extraction download failed"
                );
                analysis_api
                    .mark_failed(FailStorageObjectAnalysisRequest {
                        id: task.analysis.id,
                        error_message: error_message.clone(),
                    })
                    .await?;
                return self
                    .update_file_extraction(UpdateConversationFileExtractionRequest {
                        user_id: user_id.to_string(),
                        file_id: file_id.to_string(),
                        status: Some(EXTRACTION_STATUS_FAILED.to_string()),
                        text: None,
                        content_kind: Some("document_text".to_string()),
                        source: Some("server_file_ingest".to_string()),
                        parser: Some(FILE_INGEST_PARSER.to_string()),
                        language: None,
                        failure_reason: Some(error_message),
                        metadata: Some(json!({
                            "parserVersion": FILE_INGEST_PARSER_VERSION,
                            "analysisId": task.analysis.id,
                            "analysisKind": task.analysis.analysis_kind,
                            "storageObjectSha256": task.object.sha256,
                        })),
                    })
                    .await;
            }
        };
        tracing::debug!(
            file_id = %file.id,
            analysis_id = %task.analysis.id,
            object_key = %file.object_key,
            bytes_len = bytes.len(),
            "document text extraction downloaded object"
        );

        let source_name = file
            .file_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(file.object_key.as_str());
        let document = match file_ingest::ingest_bytes(source_name, &bytes) {
            Ok(document) => document,
            Err(err) => {
                let error_message = format!("parse failed: {err}");
                tracing::warn!(
                    file_id = %file.id,
                    analysis_id = %task.analysis.id,
                    source_name = %source_name,
                    object_key = %file.object_key,
                    bytes_len = bytes.len(),
                    error = %err,
                    "document text extraction parse failed"
                );
                analysis_api
                    .mark_failed(FailStorageObjectAnalysisRequest {
                        id: task.analysis.id,
                        error_message: error_message.clone(),
                    })
                    .await?;
                return self
                    .update_file_extraction(UpdateConversationFileExtractionRequest {
                        user_id: user_id.to_string(),
                        file_id: file_id.to_string(),
                        status: Some(EXTRACTION_STATUS_FAILED.to_string()),
                        text: None,
                        content_kind: Some("document_text".to_string()),
                        source: Some("server_file_ingest".to_string()),
                        parser: Some(FILE_INGEST_PARSER.to_string()),
                        language: None,
                        failure_reason: Some(error_message),
                        metadata: Some(json!({
                            "parserVersion": FILE_INGEST_PARSER_VERSION,
                            "analysisId": task.analysis.id,
                            "analysisKind": task.analysis.analysis_kind,
                            "storageObjectSha256": task.object.sha256,
                        })),
                    })
                    .await;
            }
        };
        tracing::debug!(
            file_id = %file.id,
            analysis_id = %task.analysis.id,
            source_name = %source_name,
            file_kind = ?document.kind,
            checksum_sha256 = ?document.checksum_sha256,
            "document text extraction parsed document"
        );

        let text = document.to_ai_text_with_options(&ViewOptions {
            max_table_rows: 100,
            max_cell_chars: 240,
        });
        let text = match normalize_required_text(text, "text") {
            Ok(text) => text,
            Err(_) => {
                let error_message = "document contains no readable text".to_string();
                tracing::warn!(
                    file_id = %file.id,
                    analysis_id = %task.analysis.id,
                    source_name = %source_name,
                    file_kind = ?document.kind,
                    "document text extraction produced empty text"
                );
                analysis_api
                    .mark_failed(FailStorageObjectAnalysisRequest {
                        id: task.analysis.id,
                        error_message: error_message.clone(),
                    })
                    .await?;
                return self
                    .update_file_extraction(UpdateConversationFileExtractionRequest {
                        user_id: user_id.to_string(),
                        file_id: file_id.to_string(),
                        status: Some(EXTRACTION_STATUS_FAILED.to_string()),
                        text: None,
                        content_kind: Some("document_text".to_string()),
                        source: Some("server_file_ingest".to_string()),
                        parser: Some(FILE_INGEST_PARSER.to_string()),
                        language: None,
                        failure_reason: Some(error_message),
                        metadata: Some(json!({
                            "parserVersion": FILE_INGEST_PARSER_VERSION,
                            "analysisId": task.analysis.id,
                            "analysisKind": task.analysis.analysis_kind,
                            "storageObjectSha256": task.object.sha256,
                        })),
                    })
                    .await;
            }
        };
        let original_text_chars = text.chars().count();
        let truncated = original_text_chars > MAX_EXTRACTED_TEXT_CHARS;
        let text = if truncated {
            text.chars().take(MAX_EXTRACTED_TEXT_CHARS).collect()
        } else {
            text
        };

        let text_chars = text.chars().count();
        tracing::debug!(
            file_id = %file.id,
            analysis_id = %task.analysis.id,
            source_name = %source_name,
            file_kind = ?document.kind,
            text_chars,
            original_text_chars,
            truncated,
            "document text extraction succeeded"
        );
        analysis_api
            .mark_succeeded(CompleteStorageObjectAnalysisRequest {
                id: task.analysis.id,
                llm_model: None,
                summary: compact_text(&text, 2000),
                result_json: json!({
                    "kind": "document_text_extraction",
                    "parser": FILE_INGEST_PARSER,
                    "parserVersion": FILE_INGEST_PARSER_VERSION,
                    "fileKind": document.kind,
                    "text": text,
                    "textChars": text_chars,
                    "originalTextChars": original_text_chars,
                    "truncated": truncated,
                    "checksumSha256": document.checksum_sha256,
                    "storageObjectSha256": task.object.sha256,
                    "sourceName": source_name,
                }),
            })
            .await?;

        self.update_file_extraction(UpdateConversationFileExtractionRequest {
            user_id: user_id.to_string(),
            file_id: file_id.to_string(),
            status: Some(EXTRACTION_STATUS_EXTRACTED.to_string()),
            text: Some(format!(
                "Document text extraction is stored in analysis {}.",
                task.analysis.id
            )),
            content_kind: Some("document_text_analysis_ref".to_string()),
            source: Some("server_file_ingest".to_string()),
            parser: Some(FILE_INGEST_PARSER.to_string()),
            language: None,
            failure_reason: None,
            metadata: Some(json!({
                "parserVersion": FILE_INGEST_PARSER_VERSION,
                "analysisId": task.analysis.id,
                "analysisKind": task.analysis.analysis_kind,
                "storageObjectSha256": task.object.sha256,
                "originalTextChars": original_text_chars,
                "truncated": truncated,
            })),
        })
        .await
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
        let updated = self
            .extract_file_text(storage, req.user_id, req.file_id)
            .await?;
        let status = build_file_extraction_status_response_from_parts(
            &updated.id,
            &updated.metadata,
            updated.created_at,
        );
        let message = match status.status.as_str() {
            EXTRACTION_STATUS_EXTRACTED => {
                "document text extraction is stored in storage_object_analyses".to_string()
            }
            EXTRACTION_STATUS_FAILED => status
                .failure_reason
                .clone()
                .unwrap_or_else(|| "document text extraction failed".to_string()),
            EXTRACTION_STATUS_UNSUPPORTED => status
                .failure_reason
                .clone()
                .unwrap_or_else(|| "document type is not supported".to_string()),
            _ => "document text extraction is pending".to_string(),
        };

        Ok(ConversationFileExtractionJobResponse {
            job_id: format!("document-text-extraction-{}", updated.id),
            file_id: status.file_id,
            status: status.status,
            content_kind: status.content_kind,
            parser: status.parser,
            source: status.source,
            message,
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
            strategy_id: item.strategy_id,
            response_mode: item.response_mode,
            routing_mode: item.routing_mode,
            collaboration_mode: item.collaboration_mode,
            learning_level: item.learning_level,
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

fn normalize_strategy_enum(
    field: &str,
    value: Option<String>,
    allowed: &[&str],
) -> BizResult<Option<String>> {
    let Some(value) = normalize_optional(value) else {
        return Ok(None);
    };
    if allowed.iter().any(|allowed| *allowed == value) {
        return Ok(Some(value));
    }
    Err(BizError::new(
        app_error::CONVERSATION_REQUEST_INVALID,
        format!("{field} has unsupported value '{value}'"),
    ))
}

fn read_strategy_string(content_json: &Option<Value>, key: &str) -> Option<String> {
    content_json
        .as_ref()
        .and_then(|value| value.get("strategy"))
        .and_then(|strategy| strategy.get(key))
        .and_then(Value::as_str)
        .map(str::to_string)
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

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn build_file_extraction_status_response(
    file: &ConversationFile,
) -> ConversationFileExtractionStatusResponse {
    build_file_extraction_status_response_from_parts(
        &uuid_to_string(file.id),
        &file.metadata,
        file.created_at,
    )
}

fn build_file_extraction_status_response_from_parts(
    file_id: &str,
    metadata: &Value,
    updated_at: OffsetDateTime,
) -> ConversationFileExtractionStatusResponse {
    let extraction = metadata.get("extraction").and_then(Value::as_object);
    let status = metadata
        .get("extractionStatus")
        .and_then(Value::as_str)
        .or_else(|| {
            extraction
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str)
        })
        .map(normalize_file_extraction_status_value)
        .unwrap_or_else(|| EXTRACTION_STATUS_PENDING.to_string());
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
    let failure_reason = extraction
        .and_then(|value| value.get("failureReason"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let text = file_extraction_inline_text(metadata);
    let text_chars = extraction
        .and_then(|value| value.get("textChars"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .or_else(|| text.as_deref().map(str::chars).map(Iterator::count))
        .unwrap_or(0);
    let vision_ready = extraction
        .and_then(|value| value.get("visionReady"))
        .and_then(Value::as_bool)
        .unwrap_or_else(|| content_kind.as_deref() == Some("image_vision_input"));
    let readable = status == EXTRACTION_STATUS_EXTRACTED
        && (text_chars > 0
            || content_kind.as_deref() == Some("document_text_analysis_ref")
            || vision_ready);

    ConversationFileExtractionStatusResponse {
        file_id: file_id.to_string(),
        status,
        content_kind,
        parser,
        source,
        language,
        summary: text.as_deref().map(|value| compact_text(value, 240)),
        text_chars,
        readable,
        vision_ready,
        failure_reason,
        metadata: metadata.clone(),
        updated_at,
    }
}

fn file_extraction_inline_text(metadata: &Value) -> Option<String> {
    metadata_text_field(metadata, &["extractedText", "textContent"])
        .or_else(|| metadata_nested_text_field(metadata, "extraction"))
}

fn metadata_nested_text_field(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|value| metadata_text_field(value, &["text"]))
}

fn metadata_text_field(metadata: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| metadata.get(*key))
        .filter_map(Value::as_str)
        .find_map(|value| normalize_optional(Some(value.to_string())))
}

fn normalize_file_extraction_status_value(value: &str) -> String {
    match value.trim() {
        "ready" | "success" | EXTRACTION_STATUS_EXTRACTED => {
            EXTRACTION_STATUS_EXTRACTED.to_string()
        }
        "error" | EXTRACTION_STATUS_FAILED => EXTRACTION_STATUS_FAILED.to_string(),
        EXTRACTION_STATUS_UNSUPPORTED => EXTRACTION_STATUS_UNSUPPORTED.to_string(),
        "queued" | "processing" | EXTRACTION_STATUS_PENDING => {
            EXTRACTION_STATUS_PENDING.to_string()
        }
        _ => EXTRACTION_STATUS_PENDING.to_string(),
    }
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
    let store_inline_text = patch.content_kind != "document_text_analysis_ref";
    object.insert(
        "extractionStatus".to_string(),
        Value::String(patch.status.clone()),
    );
    if let Some(text) = patch.text.as_ref().filter(|_| store_inline_text) {
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
    if let Some(text) = patch.text.filter(|_| store_inline_text) {
        extraction.insert("text".to_string(), Value::String(text));
    } else {
        extraction.remove("text");
    }
    extraction.insert(
        "textChars".to_string(),
        Value::Number(serde_json::Number::from(patch.text_chars)),
    );
    merge_extraction_extra_metadata(&mut extraction, patch.extra_metadata);
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
    object.insert("extraction".to_string(), Value::Object(extraction));

    Value::Object(object)
}

fn merge_extraction_extra_metadata(extraction: &mut Map<String, Value>, extra_metadata: Value) {
    let Value::Object(mut extra_metadata) = extra_metadata else {
        extraction.insert("metadata".to_string(), json!({}));
        return;
    };
    for key in [
        "parserVersion",
        "fileKind",
        "checksumSha256",
        "originalTextChars",
        "truncated",
        "analysisId",
        "analysisKind",
        "storageObjectSha256",
    ] {
        if let Some(value) = extra_metadata.remove(key) {
            extraction.insert(key.to_string(), value);
        } else {
            extraction.remove(key);
        }
    }
    extraction.insert("metadata".to_string(), Value::Object(extra_metadata));
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
    fn conversation_file_analysis_ref_does_not_store_inline_text() {
        let metadata = build_file_extraction_metadata(
            json!({}),
            FileExtractionMetadataPatch {
                status: EXTRACTION_STATUS_EXTRACTED.to_string(),
                text: Some("正文保存在 analysis.result_json.text".to_string()),
                content_kind: "document_text_analysis_ref".to_string(),
                source: "server_file_ingest".to_string(),
                parser: "file-ingest".to_string(),
                language: None,
                failure_reason: None,
                text_chars: 0,
                extra_metadata: json!({
                    "analysisId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "analysisKind": "document_text_extraction",
                    "storageObjectSha256": "sha256",
                }),
            },
        );

        assert_eq!(metadata["extractionStatus"], EXTRACTION_STATUS_EXTRACTED);
        assert_eq!(
            metadata["extraction"]["contentKind"],
            "document_text_analysis_ref"
        );
        assert_eq!(
            metadata["extraction"]["analysisKind"],
            "document_text_extraction"
        );
        assert!(metadata.get("extractedText").is_none());
        assert!(metadata.get("textContent").is_none());
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
}
