use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::platform_connections,
    table::platform_connections::dto::{
        PlatformConnection, PlatformConnectionFilter, PlatformConnectionStatus, PlatformSyncStatus,
        UpdatePlatformConnection, UpsertPlatformConnection,
    },
};

db_core::impl_repository!(
    PlatformConnectionRepo,
    platform_connections::Entity,
    platform_connections::Model
);

pub struct PlatformConnectionService {
    repo: PlatformConnectionRepo,
}

impl PlatformConnectionService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: PlatformConnectionRepo::new(db),
        }
    }

    pub async fn upsert(&self, input: UpsertPlatformConnection) -> BizResult<PlatformConnection> {
        if let Some(existing) = self
            .get_by_user_platform(input.user_id, input.platform.as_str())
            .await?
        {
            return self
                .update(UpdatePlatformConnection {
                    id: existing.id,
                    platform: input.platform,
                    app_key: input.app_key,
                    app_secret_enc: input.app_secret_enc,
                    access_token_enc: input.access_token_enc,
                    refresh_token_enc: input.refresh_token_enc,
                    token_expires_at: input.token_expires_at,
                    shop_id: input.shop_id,
                    shop_name: input.shop_name,
                    enabled: input.enabled,
                    status: input.status,
                    last_tested_at: input.last_tested_at,
                    test_result: input.test_result,
                    last_synced_at: input.last_synced_at,
                    sync_status: input.sync_status,
                    sync_message: input.sync_message,
                    extra: input.extra,
                })
                .await;
        }

        let model = platform_connections::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            platform: Set(input.platform),
            app_key: Set(input.app_key),
            app_secret_enc: Set(input.app_secret_enc),
            access_token_enc: Set(input.access_token_enc),
            refresh_token_enc: Set(input.refresh_token_enc),
            token_expires_at: Set(input.token_expires_at),
            shop_id: Set(input.shop_id),
            shop_name: Set(input.shop_name),
            enabled: Set(input.enabled),
            status: Set(input.status.as_str().to_string()),
            last_tested_at: Set(input.last_tested_at),
            test_result: Set(input.test_result),
            last_synced_at: Set(input.last_synced_at),
            sync_status: Set(input.sync_status.as_str().to_string()),
            sync_message: Set(input.sync_message),
            extra: Set(input.extra),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<PlatformConnection>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_user_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> BizResult<Option<PlatformConnection>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(platform_connections::Column::UserId.eq(user_id))
                    .filter(platform_connections::Column::Platform.eq(platform)),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(
        &self,
        filter: PlatformConnectionFilter,
    ) -> BizResult<Vec<PlatformConnection>> {
        let mut query = self
            .repo
            .query()
            .filter(platform_connections::Column::UserId.eq(filter.user_id));

        if let Some(platform) = filter.platform {
            query = query.filter(platform_connections::Column::Platform.eq(platform));
        }

        if let Some(enabled) = filter.enabled {
            query = query.filter(platform_connections::Column::Enabled.eq(enabled));
        }

        query = query.order_by_asc(platform_connections::Column::Platform);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn update(&self, input: UpdatePlatformConnection) -> BizResult<PlatformConnection> {
        let model = platform_connections::ActiveModel {
            id: Unchanged(input.id),
            platform: Set(input.platform),
            app_key: Set(input.app_key),
            app_secret_enc: Set(input.app_secret_enc),
            access_token_enc: Set(input.access_token_enc),
            refresh_token_enc: Set(input.refresh_token_enc),
            token_expires_at: Set(input.token_expires_at),
            shop_id: Set(input.shop_id),
            shop_name: Set(input.shop_name),
            enabled: Set(input.enabled),
            status: Set(input.status.as_str().to_string()),
            last_tested_at: Set(input.last_tested_at),
            test_result: Set(input.test_result),
            last_synced_at: Set(input.last_synced_at),
            sync_status: Set(input.sync_status.as_str().to_string()),
            sync_message: Set(input.sync_message),
            extra: Set(input.extra),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: platform_connections::Model) -> BizResult<PlatformConnection> {
        Ok(PlatformConnection {
            id: model.id,
            user_id: model.user_id,
            platform: model.platform,
            app_key: model.app_key,
            app_secret_enc: model.app_secret_enc,
            access_token_enc: model.access_token_enc,
            refresh_token_enc: model.refresh_token_enc,
            token_expires_at: model.token_expires_at,
            shop_id: model.shop_id,
            shop_name: model.shop_name,
            enabled: model.enabled,
            status: PlatformConnectionStatus::try_from(model.status.as_str())?,
            last_tested_at: model.last_tested_at,
            test_result: model.test_result,
            last_synced_at: model.last_synced_at,
            sync_status: PlatformSyncStatus::try_from(model.sync_status.as_str())?,
            sync_message: model.sync_message,
            extra: model.extra,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
