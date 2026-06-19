use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::account_usage_quotas,
    table::account_usage_quotas::dto::{
        AccountUsageQuota, AccountUsageQuotaStatus, CreateAccountUsageQuota,
        UpdateAccountUsageQuota,
    },
};

db_core::impl_repository!(
    AccountUsageQuotaRepo,
    account_usage_quotas::Entity,
    account_usage_quotas::Model
);

pub struct AccountUsageQuotaService {
    repo: AccountUsageQuotaRepo,
}

impl AccountUsageQuotaService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AccountUsageQuotaRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAccountUsageQuota) -> BizResult<AccountUsageQuota> {
        let model = account_usage_quotas::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            question_limit: Set(input.question_limit),
            status: Set(input.status.as_str().to_owned()),
            remark: Set(input.remark),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list_all(&self) -> BizResult<Vec<AccountUsageQuota>> {
        let query = self
            .repo
            .query()
            .order_by_asc(account_usage_quotas::Column::UpdatedAt);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<AccountUsageQuota>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_user_id(&self, user_id: Uuid) -> BizResult<Option<AccountUsageQuota>> {
        let query = self
            .repo
            .query()
            .filter(account_usage_quotas::Column::UserId.eq(user_id));

        self.repo
            .select_one(query)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn update(&self, input: UpdateAccountUsageQuota) -> BizResult<AccountUsageQuota> {
        let model = account_usage_quotas::ActiveModel {
            id: Unchanged(input.id),
            question_limit: Set(input.question_limit),
            status: Set(input.status.as_str().to_owned()),
            remark: Set(input.remark),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: account_usage_quotas::Model) -> BizResult<AccountUsageQuota> {
        Ok(AccountUsageQuota {
            id: model.id,
            user_id: model.user_id,
            question_limit: model.question_limit,
            status: AccountUsageQuotaStatus::try_from(model.status)?,
            remark: model.remark,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
