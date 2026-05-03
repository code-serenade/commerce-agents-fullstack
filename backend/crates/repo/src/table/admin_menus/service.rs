use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, QueryOrder};

use crate::{
    entity::admin_menus,
    table::admin_menus::dto::{CreateMenu, Menu},
};

db_core::impl_repository!(MenuRepo, admin_menus::Entity, admin_menus::Model);

pub struct MenuService {
    repo: MenuRepo,
}

impl MenuService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: MenuRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateMenu) -> BizResult<Menu> {
        let model = admin_menus::ActiveModel {
            name: Set(input.name),
            parent_id: Set(input.parent_id),
            permission_code: Set(input.permission_code),
            ..Default::default()
        };

        Ok(Self::from_model(self.repo.insert(model).await?))
    }

    pub async fn list_all(&self) -> BizResult<Vec<Menu>> {
        let query = self.repo.query().order_by_asc(admin_menus::Column::Id);
        Ok(self
            .repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect())
    }

    fn from_model(model: admin_menus::Model) -> Menu {
        Menu {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            permission_code: model.permission_code,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
