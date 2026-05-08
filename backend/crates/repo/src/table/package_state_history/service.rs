use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{
    entity::package_state_history,
    table::{
        package_registry::{PackageStatus, PackageType},
        package_state_history::dto::{
            CreatePackageStateHistory, PackageStateHistory, PackageStateHistoryFilter,
        },
    },
};

db_core::impl_repository!(
    PackageStateHistoryRepo,
    package_state_history::Entity,
    package_state_history::Model
);

pub struct PackageStateHistoryService {
    repo: PackageStateHistoryRepo,
}

impl PackageStateHistoryService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: PackageStateHistoryRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreatePackageStateHistory) -> BizResult<PackageStateHistory> {
        let model = package_state_history::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            package_registry_id: Set(input.package_registry_id),
            package_type: Set(input.package_type.as_str().to_string()),
            package_id: Set(input.package_id),
            from_status: Set(input.from_status.as_str().to_string()),
            to_status: Set(input.to_status.as_str().to_string()),
            reason: Set(input.reason),
            changed_by: Set(input.changed_by),
            metadata: Set(input.metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list(
        &self,
        filter: PackageStateHistoryFilter,
    ) -> BizResult<Vec<PackageStateHistory>> {
        let mut query = self
            .repo
            .query()
            .filter(package_state_history::Column::UserId.eq(filter.user_id));

        if let Some(package_type) = filter.package_type {
            query =
                query.filter(package_state_history::Column::PackageType.eq(package_type.as_str()));
        }

        if let Some(package_id) = filter.package_id {
            query = query.filter(package_state_history::Column::PackageId.eq(package_id));
        }

        query = query.order_by_desc(package_state_history::Column::CreatedAt);

        if let Some(offset) = filter.offset {
            query = query.offset(offset);
        }

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    fn from_model(model: package_state_history::Model) -> BizResult<PackageStateHistory> {
        Ok(PackageStateHistory {
            id: model.id,
            user_id: model.user_id,
            package_registry_id: model.package_registry_id,
            package_type: PackageType::try_from(model.package_type.as_str())?,
            package_id: model.package_id,
            from_status: PackageStatus::try_from(model.from_status.as_str())?,
            to_status: PackageStatus::try_from(model.to_status.as_str())?,
            reason: model.reason,
            changed_by: model.changed_by,
            metadata: model.metadata,
            created_at: model.created_at,
        })
    }
}
