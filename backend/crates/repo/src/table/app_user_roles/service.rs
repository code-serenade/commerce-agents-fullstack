use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, QueryFilter, QuerySelect,
    sea_query::{ExprTrait, IntoCondition},
};
use uuid::Uuid;

use crate::{
    entity::app_user_roles,
    table::app_user_roles::dto::{CreateUserRole, UserRole},
};

db_core::impl_repository!(UserRoleRepo, app_user_roles::Entity, app_user_roles::Model);

pub struct UserRoleService {
    repo: UserRoleRepo,
}

impl UserRoleService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: UserRoleRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateUserRole) -> BizResult<UserRole> {
        let model = app_user_roles::ActiveModel {
            user_id: Set(input.user_id),
            role_id: Set(input.role_id),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn list_by_user_id(&self, user_id: Uuid) -> BizResult<Vec<UserRole>> {
        let query = self
            .repo
            .query()
            .filter(app_user_roles::Column::UserId.eq(user_id));

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn list_by_user_ids(&self, user_ids: Vec<Uuid>) -> BizResult<Vec<UserRole>> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = self
            .repo
            .query()
            .filter(app_user_roles::Column::UserId.is_in(user_ids));

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn count_multi_role_users(&self) -> BizResult<u64> {
        let user_ids = self
            .repo
            .query()
            .select_only()
            .column(app_user_roles::Column::UserId)
            .group_by(app_user_roles::Column::UserId)
            .having(app_user_roles::Column::RoleId.count().gt(1))
            .into_tuple::<Uuid>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(user_ids.len() as u64)
    }

    pub async fn exists(&self, user_id: Uuid, role_id: i64) -> BizResult<bool> {
        let query = self
            .repo
            .query()
            .filter(app_user_roles::Column::UserId.eq(user_id))
            .filter(app_user_roles::Column::RoleId.eq(role_id));

        Ok(self.repo.select_one(query).await?.is_some())
    }

    pub async fn delete_by_user_id(&self, user_id: Uuid) -> BizResult<u64> {
        Ok(self
            .repo
            .delete_many(app_user_roles::Column::UserId.eq(user_id).into_condition())
            .await?
            .rows_affected)
    }

    pub async fn delete_by_role_id(&self, role_id: i64) -> BizResult<u64> {
        Ok(self
            .repo
            .delete_many(app_user_roles::Column::RoleId.eq(role_id).into_condition())
            .await?
            .rows_affected)
    }

    fn from_model(model: app_user_roles::Model) -> UserRole {
        UserRole {
            user_id: model.user_id,
            role_id: model.role_id,
            created_at: model.created_at,
        }
    }
}
