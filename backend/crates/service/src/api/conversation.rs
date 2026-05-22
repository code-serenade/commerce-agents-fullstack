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
use storage::{BucketKind, DeleteObjectRequest, ObjectStorage, PresignDownloadRequest};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::dto::conversation::{
    AddConversationMessageRequest, ConversationDetailResponse, ConversationFileAccessResponse,
    ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
    CreateConversationRequest, DeleteConversationFileRequest, DeleteConversationRequest,
    RegisterConversationFileRequest, UpdateConversationFileExtractionRequest,
    UpdateConversationRequest, uuid_to_string,
};

const DEFAULT_LIST_LIMIT: u64 = 50;
const FILE_ACCESS_EXPIRES_SECS: u64 = 600;
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

    pub async fn delete_conversation(&self, req: DeleteConversationRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_active_conversation_owner(user_id, conversation_id)
            .await?;

        self.conversation_svc
            .update(repo::table::conversations::UpdateConversation {
                id: conversation.id,
                title: conversation.title,
                status: "deleted".to_string(),
                archived_at: Some(OffsetDateTime::now_utc()),
            })
            .await?;

        Ok(())
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

        let file = self
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
        Ok(Self::map_file(file))
    }

    pub async fn delete_file(
        &self,
        storage: &ObjectStorage,
        req: DeleteConversationFileRequest,
    ) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let file = self.ensure_active_file_owner(user_id, file_id).await?;
        self.ensure_active_conversation_owner(user_id, file.conversation_id)
            .await?;

        storage
            .delete_object(DeleteObjectRequest {
                bucket: bucket_kind_from_name(&file.bucket, storage)?,
                key: file.object_key.clone(),
            })
            .await
            .map_err(|err| {
                BizError::new(app_error::CONVERSATION_REQUEST_INVALID, err.to_string())
            })?;

        self.file_svc
            .update_status(UpdateConversationFileStatus {
                id: file.id,
                status: "deleted".to_string(),
                deleted_at: Some(OffsetDateTime::now_utc()),
            })
            .await?;

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

fn bucket_kind_from_name(bucket: &str, storage: &ObjectStorage) -> BizResult<BucketKind> {
    let config = storage.config();
    if bucket == "private" || bucket == config.private_bucket {
        Ok(BucketKind::Private)
    } else if bucket == "public" || bucket == config.public_bucket {
        Ok(BucketKind::Public)
    } else {
        Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "invalid bucket".to_string(),
        ))
    }
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
}
