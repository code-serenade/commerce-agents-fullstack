use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    conversation_files::{ConversationFile, ConversationFileService, CreateConversationFile},
    conversation_messages::{
        ConversationMessage, ConversationMessageService, CreateConversationMessage,
    },
    conversations::{Conversation, ConversationService, CreateConversation},
};
use serde_json::json;
use storage::{BucketKind, ObjectStorage, PresignDownloadRequest};
use uuid::Uuid;

use crate::dto::conversation::{
    AddConversationMessageRequest, ConversationDetailResponse, ConversationFileAccessResponse,
    ConversationFileResponse, ConversationMessageResponse, ConversationResponse,
    CreateConversationRequest, RegisterConversationFileRequest, uuid_to_string,
};

const DEFAULT_LIST_LIMIT: u64 = 50;
const FILE_ACCESS_EXPIRES_SECS: u64 = 600;

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

    pub async fn register_file(
        &self,
        req: RegisterConversationFileRequest,
    ) -> BizResult<ConversationFileResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        self.ensure_conversation_owner(user_id, conversation_id)
            .await?;
        let message_id = req
            .message_id
            .as_deref()
            .map(|value| parse_uuid(value, "message_id"))
            .transpose()?;

        let file = self
            .file_svc
            .create(CreateConversationFile {
                id: Uuid::new_v4(),
                user_id,
                conversation_id,
                message_id,
                bucket: "private".to_string(),
                object_key: req.object_key,
                file_name: normalize_optional(req.file_name),
                mime_type: normalize_optional(req.mime_type),
                file_size: req.file_size,
                sha256: normalize_optional(req.sha256),
                metadata: json!({}),
            })
            .await?;
        Ok(Self::map_file(file))
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
