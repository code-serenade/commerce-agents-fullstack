use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder};

use crate::{
    entity::app_permissions,
    table::app_permissions::dto::{CreatePermission, Permission, PermissionKind},
};

db_core::impl_repository!(
    PermissionRepo,
    app_permissions::Entity,
    app_permissions::Model
);

pub struct PermissionService {
    repo: PermissionRepo,
}

impl PermissionService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: PermissionRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreatePermission) -> BizResult<Permission> {
        let model = app_permissions::ActiveModel {
            code: Set(input.code),
            name: Set(input.name),
            parent_code: Set(input.parent_code),
            sort: Set(input.sort),
            kind: Set(input.kind.as_str().to_owned()),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list_all(&self) -> BizResult<Vec<Permission>> {
        let query = self.repo.query().order_by_asc(app_permissions::Column::Id);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn get_by_code(&self, code: &str) -> BizResult<Option<Permission>> {
        let query = self
            .repo
            .query()
            .filter(app_permissions::Column::Code.eq(code.to_owned()));

        self.repo
            .select_one(query)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list_by_ids(&self, ids: Vec<i64>) -> BizResult<Vec<Permission>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = self
            .repo
            .query()
            .filter(app_permissions::Column::Id.is_in(ids))
            .order_by_asc(app_permissions::Column::Id);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    fn from_model(model: app_permissions::Model) -> BizResult<Permission> {
        Ok(Permission {
            id: model.id,
            code: model.code,
            name: model.name,
            parent_code: model.parent_code,
            sort: model.sort,
            kind: PermissionKind::try_from(model.kind)?,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
