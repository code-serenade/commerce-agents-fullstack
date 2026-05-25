use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{ActiveEnum, ActiveValue::Set, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    entity::{
        conversation_messages,
        sea_orm_active_enums::{
            ConversationCollaborationMode, ConversationLearningLevel, ConversationResponseMode,
            ConversationRoutingMode, ConversationStrategyId,
        },
    },
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
            strategy_id: Set(input.strategy_id.map(strategy_id_from_string)),
            response_mode: Set(input.response_mode.map(response_mode_from_string)),
            routing_mode: Set(input.routing_mode.map(routing_mode_from_string)),
            collaboration_mode: Set(input.collaboration_mode.map(collaboration_mode_from_string)),
            learning_level: Set(input.learning_level.map(learning_level_from_string)),
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
            strategy_id: model.strategy_id.map(|value| value.to_value()),
            response_mode: model.response_mode.map(|value| value.to_value()),
            routing_mode: model.routing_mode.map(|value| value.to_value()),
            collaboration_mode: model.collaboration_mode.map(|value| value.to_value()),
            learning_level: model.learning_level.map(|value| value.to_value()),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

fn strategy_id_from_string(value: String) -> ConversationStrategyId {
    match value.as_str() {
        "learning" => ConversationStrategyId::Learning,
        "quick" => ConversationStrategyId::Quick,
        "execution" => ConversationStrategyId::Execution,
        _ => unreachable!("strategy_id is validated before repo insert"),
    }
}

fn response_mode_from_string(value: String) -> ConversationResponseMode {
    match value.as_str() {
        "learning" => ConversationResponseMode::Learning,
        "quick" => ConversationResponseMode::Quick,
        "execution" => ConversationResponseMode::Execution,
        _ => unreachable!("response_mode is validated before repo insert"),
    }
}

fn routing_mode_from_string(value: String) -> ConversationRoutingMode {
    match value.as_str() {
        "commander" => ConversationRoutingMode::Commander,
        "fast" => ConversationRoutingMode::Fast,
        _ => unreachable!("routing_mode is validated before repo insert"),
    }
}

fn collaboration_mode_from_string(value: String) -> ConversationCollaborationMode {
    match value.as_str() {
        "auto" => ConversationCollaborationMode::Auto,
        "single" => ConversationCollaborationMode::Single,
        "manual" => ConversationCollaborationMode::Manual,
        _ => unreachable!("collaboration_mode is validated before repo insert"),
    }
}

fn learning_level_from_string(value: String) -> ConversationLearningLevel {
    match value.as_str() {
        "beginner" => ConversationLearningLevel::Beginner,
        "higher_vocational" => ConversationLearningLevel::HigherVocational,
        "professional" => ConversationLearningLevel::Professional,
        _ => unreachable!("learning_level is validated before repo insert"),
    }
}
