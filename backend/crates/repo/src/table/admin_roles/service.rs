use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder};

use crate::{
    entity::admin_roles,
    table::admin_roles::dto::{CreateRole, Role},
};

db_core::impl_repository!(RoleRepo, admin_roles::Entity, admin_roles::Model);

pub struct RoleService {
    repo: RoleRepo,
}

impl RoleService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: RoleRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateRole) -> BizResult<Role> {
        let model = admin_roles::ActiveModel {
            name: Set(input.name),
            code: Set(input.code),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn list_all(&self) -> BizResult<Vec<Role>> {
        let query = self.repo.query().order_by_asc(admin_roles::Column::Id);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn list_by_ids(&self, ids: Vec<i64>) -> BizResult<Vec<Role>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = self
            .repo
            .query()
            .filter(admin_roles::Column::Id.is_in(ids))
            .order_by_asc(admin_roles::Column::Id);

        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    pub async fn get_by_id(&self, id: i64) -> BizResult<Option<Role>> {
        Ok(self.repo.find_by_id(id).await?.map(Self::from_model))
    }

    pub async fn delete_by_id(&self, id: i64) -> BizResult<u64> {
        Ok(self.repo.delete_by_id(id).await?.rows_affected)
    }

    fn from_model(model: admin_roles::Model) -> Role {
        Role {
            id: model.id,
            name: model.name,
            code: model.code,
            created_at: model.created_at,
        }
    }
}
