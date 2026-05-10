use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::conversation_files,
    table::conversation_files::dto::{
        ConversationFile, CreateConversationFile, UpdateConversationFileMetadata,
    },
};

db_core::impl_repository!(
    ConversationFileRepo,
    conversation_files::Entity,
    conversation_files::Model
);

pub struct ConversationFileService {
    repo: ConversationFileRepo,
}

impl ConversationFileService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ConversationFileRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateConversationFile) -> BizResult<ConversationFile> {
        let model = conversation_files::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            conversation_id: Set(input.conversation_id),
            message_id: Set(input.message_id),
            bucket: Set(input.bucket),
            object_key: Set(input.object_key),
            file_name: Set(input.file_name),
            mime_type: Set(input.mime_type),
            file_size: Set(input.file_size),
            sha256: Set(input.sha256),
            visibility: Set("private".to_string()),
            storage_provider: Set("s3".to_string()),
            status: Set("active".to_string()),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ConversationFile>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn list_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> BizResult<Vec<ConversationFile>> {
        let query = self
            .repo
            .query()
            .filter(conversation_files::Column::ConversationId.eq(conversation_id))
            .filter(conversation_files::Column::Status.eq("active"))
            .order_by_asc(conversation_files::Column::CreatedAt);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn update_metadata(
        &self,
        input: UpdateConversationFileMetadata,
    ) -> BizResult<ConversationFile> {
        let model = conversation_files::ActiveModel {
            id: Unchanged(input.id),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.update(model).await?))
    }

    fn from_model(model: conversation_files::Model) -> ConversationFile {
        ConversationFile {
            id: model.id,
            user_id: model.user_id,
            conversation_id: model.conversation_id,
            message_id: model.message_id,
            bucket: model.bucket,
            object_key: model.object_key,
            file_name: model.file_name,
            mime_type: model.mime_type,
            file_size: model.file_size,
            sha256: model.sha256,
            visibility: model.visibility,
            storage_provider: model.storage_provider,
            status: model.status,
            metadata: model.metadata,
            created_at: model.created_at,
            deleted_at: model.deleted_at,
        }
    }
}
