use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, sea_query::IntoCondition};
use uuid::Uuid;

use crate::{
    entity::admin_user_roles,
    table::admin_user_roles::dto::{CreateUserRole, UserRole},
};

db_core::impl_repository!(
    UserRoleRepo,
    admin_user_roles::Entity,
    admin_user_roles::Model
);

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
        let model = admin_user_roles::ActiveModel {
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
            .filter(admin_user_roles::Column::UserId.eq(user_id));

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
            .filter(admin_user_roles::Column::UserId.is_in(user_ids));

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn delete_by_user_id(&self, user_id: Uuid) -> BizResult<u64> {
        Ok(self
            .repo
            .delete_many(
                admin_user_roles::Column::UserId
                    .eq(user_id)
                    .into_condition(),
            )
            .await?
            .rows_affected)
    }

    pub async fn delete_by_role_id(&self, role_id: i64) -> BizResult<u64> {
        Ok(self
            .repo
            .delete_many(
                admin_user_roles::Column::RoleId
                    .eq(role_id)
                    .into_condition(),
            )
            .await?
            .rows_affected)
    }

    fn from_model(model: admin_user_roles::Model) -> UserRole {
        UserRole {
            user_id: model.user_id,
            role_id: model.role_id,
            created_at: model.created_at,
        }
    }
}
