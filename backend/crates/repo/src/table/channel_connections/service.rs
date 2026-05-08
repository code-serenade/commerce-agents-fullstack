use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::channel_connections,
    table::channel_connections::dto::{
        ChannelConnection, ChannelConnectionFilter, ChannelConnectionStatus,
        UpdateChannelConnection, UpsertChannelConnection,
    },
};

db_core::impl_repository!(
    ChannelConnectionRepo,
    channel_connections::Entity,
    channel_connections::Model
);

pub struct ChannelConnectionService {
    repo: ChannelConnectionRepo,
}

impl ChannelConnectionService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: ChannelConnectionRepo::new(db),
        }
    }

    pub async fn upsert(&self, input: UpsertChannelConnection) -> BizResult<ChannelConnection> {
        if let Some(existing) = self
            .get_by_user_channel(input.user_id, input.channel.as_str())
            .await?
        {
            return self
                .update(UpdateChannelConnection {
                    id: existing.id,
                    channel: input.channel,
                    webhook_url: input.webhook_url,
                    bot_token_enc: input.bot_token_enc,
                    chat_id: input.chat_id,
                    secret_enc: input.secret_enc,
                    extra_credentials: input.extra_credentials,
                    enabled: input.enabled,
                    status: input.status,
                    last_tested_at: input.last_tested_at,
                    last_error: input.last_error,
                })
                .await;
        }

        let model = channel_connections::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            channel: Set(input.channel),
            webhook_url: Set(input.webhook_url),
            bot_token_enc: Set(input.bot_token_enc),
            chat_id: Set(input.chat_id),
            secret_enc: Set(input.secret_enc),
            extra_credentials: Set(input.extra_credentials),
            enabled: Set(input.enabled),
            status: Set(input.status.as_str().to_string()),
            last_tested_at: Set(input.last_tested_at),
            last_error: Set(input.last_error),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<ChannelConnection>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_user_channel(
        &self,
        user_id: Uuid,
        channel: &str,
    ) -> BizResult<Option<ChannelConnection>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(channel_connections::Column::UserId.eq(user_id))
                    .filter(channel_connections::Column::Channel.eq(channel)),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: ChannelConnectionFilter) -> BizResult<Vec<ChannelConnection>> {
        let mut query = self
            .repo
            .query()
            .filter(channel_connections::Column::UserId.eq(filter.user_id));

        if let Some(channel) = filter.channel {
            query = query.filter(channel_connections::Column::Channel.eq(channel));
        }

        if let Some(enabled) = filter.enabled {
            query = query.filter(channel_connections::Column::Enabled.eq(enabled));
        }

        query = query.order_by_asc(channel_connections::Column::Channel);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn update(&self, input: UpdateChannelConnection) -> BizResult<ChannelConnection> {
        let model = channel_connections::ActiveModel {
            id: Unchanged(input.id),
            channel: Set(input.channel),
            webhook_url: Set(input.webhook_url),
            bot_token_enc: Set(input.bot_token_enc),
            chat_id: Set(input.chat_id),
            secret_enc: Set(input.secret_enc),
            extra_credentials: Set(input.extra_credentials),
            enabled: Set(input.enabled),
            status: Set(input.status.as_str().to_string()),
            last_tested_at: Set(input.last_tested_at),
            last_error: Set(input.last_error),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: channel_connections::Model) -> BizResult<ChannelConnection> {
        Ok(ChannelConnection {
            id: model.id,
            user_id: model.user_id,
            channel: model.channel,
            webhook_url: model.webhook_url,
            bot_token_enc: model.bot_token_enc,
            chat_id: model.chat_id,
            secret_enc: model.secret_enc,
            extra_credentials: model.extra_credentials,
            enabled: model.enabled,
            status: ChannelConnectionStatus::try_from(model.status.as_str())?,
            last_tested_at: model.last_tested_at,
            last_error: model.last_error,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
