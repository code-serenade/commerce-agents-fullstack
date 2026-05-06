use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    entity::conversation_messages,
    table::conversation_messages::dto::{ConversationMessage, CreateConversationMessage},
};

db_core::impl_repository!(
    ConversationMessageRepo,
    conversation_messages::Entity,
    conversation_messages::Model
);

pub struct ConversationMessageService {
    repo: ConversationMessageRepo,
}

impl ConversationMessageService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ConversationMessageRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateConversationMessage) -> BizResult<ConversationMessage> {
        let model = conversation_messages::ActiveModel {
            id: Set(input.id),
            conversation_id: Set(input.conversation_id),
            user_id: Set(input.user_id),
            role: Set(input.role),
            content_text: Set(input.content_text),
            content_json: Set(input.content_json),
            status: Set("completed".to_string()),
            sequence: Set(input.sequence),
            token_count: Set(input.token_count),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn list_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> BizResult<Vec<ConversationMessage>> {
        let query = self
            .repo
            .query()
            .filter(conversation_messages::Column::ConversationId.eq(conversation_id))
            .order_by_asc(conversation_messages::Column::Sequence);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn next_sequence(&self, conversation_id: Uuid) -> BizResult<i64> {
        let count = self
            .repo
            .query()
            .filter(conversation_messages::Column::ConversationId.eq(conversation_id))
            .count(self.repo.db())
            .await
            .map_err(Error::from)?;
        Ok(count as i64 + 1)
    }

    fn from_model(model: conversation_messages::Model) -> ConversationMessage {
        ConversationMessage {
            id: model.id,
            conversation_id: model.conversation_id,
            user_id: model.user_id,
            role: model.role,
            content_text: model.content_text,
            content_json: model.content_json,
            status: model.status,
            sequence: model.sequence,
            token_count: model.token_count,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
