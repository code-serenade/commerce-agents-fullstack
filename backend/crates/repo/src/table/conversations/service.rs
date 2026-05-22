use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::conversations,
    table::conversations::dto::{
        Conversation, CreateConversation, UpdateConversation, UpdateConversationMetadata,
    },
};

db_core::impl_repository!(
    ConversationRepo,
    conversations::Entity,
    conversations::Model
);

pub struct ConversationService {
    repo: ConversationRepo,
}

impl ConversationService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ConversationRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateConversation) -> BizResult<Conversation> {
        let model = conversations::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            title: Set(input.title),
            status: Set("active".to_string()),
            source: Set(input.source),
            agent_id: Set(input.agent_id),
            model_name: Set(input.model_name),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<Conversation>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn list_by_user_id(&self, user_id: Uuid, limit: u64) -> BizResult<Vec<Conversation>> {
        let query = self
            .repo
            .query()
            .filter(conversations::Column::UserId.eq(user_id))
            .filter(conversations::Column::Status.ne("deleted"))
            .order_by_desc(conversations::Column::UpdatedAt)
            .limit(limit);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn update(&self, input: UpdateConversation) -> BizResult<Conversation> {
        let model = conversations::ActiveModel {
            id: Unchanged(input.id),
            title: Set(input.title),
            status: Set(input.status),
            archived_at: Set(input.archived_at),
            ..Default::default()
        };
        Ok(Self::from_model(self.repo.update(model).await?))
    }

    pub async fn delete_by_id(&self, id: Uuid) -> BizResult<u64> {
        Ok(self.repo.delete_by_id(id).await?.rows_affected)
    }

    pub async fn update_metadata(
        &self,
        input: UpdateConversationMetadata,
    ) -> BizResult<Conversation> {
        let model = conversations::ActiveModel {
            id: Unchanged(input.id),
            metadata: Set(input.metadata),
            ..Default::default()
        };
        Ok(Self::from_model(self.repo.update(model).await?))
    }

    fn from_model(model: conversations::Model) -> Conversation {
        Conversation {
            id: model.id,
            user_id: model.user_id,
            title: model.title,
            status: model.status,
            source: model.source,
            agent_id: model.agent_id,
            model_name: model.model_name,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
            archived_at: model.archived_at,
        }
    }
}
