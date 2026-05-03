use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, sea_query::IntoCondition};

use crate::{
    entity::admin_role_permissions,
    table::admin_role_permissions::dto::{CreateRolePermission, RolePermission},
};

db_core::impl_repository!(
    RolePermissionRepo,
    admin_role_permissions::Entity,
    admin_role_permissions::Model
);

pub struct RolePermissionService {
    repo: RolePermissionRepo,
}

impl RolePermissionService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: RolePermissionRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateRolePermission) -> BizResult<RolePermission> {
        let model = admin_role_permissions::ActiveModel {
            role_id: Set(input.role_id),
            permission_id: Set(input.permission_id),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn list_by_role_ids(&self, role_ids: Vec<i64>) -> BizResult<Vec<RolePermission>> {
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = self
            .repo
            .query()
            .filter(admin_role_permissions::Column::RoleId.is_in(role_ids));

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn delete_by_role_id(&self, role_id: i64) -> BizResult<u64> {
        Ok(self
            .repo
            .delete_many(
                admin_role_permissions::Column::RoleId
                    .eq(role_id)
                    .into_condition(),
            )
            .await?
            .rows_affected)
    }

    fn from_model(model: admin_role_permissions::Model) -> RolePermission {
        RolePermission {
            role_id: model.role_id,
            permission_id: model.permission_id,
            created_at: model.created_at,
        }
    }
}
