use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entity::package_registry,
    table::package_registry::dto::{
        PackageRegistry, PackageRegistryFilter, PackageStatus, PackageType, UpdatePackageRegistry,
        UpsertPackageRegistry,
    },
};

db_core::impl_repository!(
    PackageRegistryRepo,
    package_registry::Entity,
    package_registry::Model
);

pub struct PackageRegistryService {
    repo: PackageRegistryRepo,
}

impl PackageRegistryService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: PackageRegistryRepo::new(db),
        }
    }

    pub async fn upsert(&self, input: UpsertPackageRegistry) -> BizResult<PackageRegistry> {
        if let Some(existing) = self
            .get_by_user_package(input.user_id, input.package_type, input.package_id.as_str())
            .await?
        {
            return self
                .update(UpdatePackageRegistry {
                    id: existing.id,
                    package_name: input.package_name,
                    version: input.version,
                    status: input.status,
                    manifest_path: input.manifest_path,
                    dependencies: input.dependencies,
                    compatibility: input.compatibility,
                    metadata: input.metadata,
                    updated_by: input.updated_by,
                })
                .await;
        }

        let model = package_registry::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            package_type: Set(input.package_type.as_str().to_string()),
            package_id: Set(input.package_id),
            package_name: Set(input.package_name),
            version: Set(input.version),
            status: Set(input.status.as_str().to_string()),
            manifest_path: Set(input.manifest_path),
            dependencies: Set(input.dependencies),
            compatibility: Set(input.compatibility),
            metadata: Set(input.metadata),
            updated_by: Set(input.updated_by),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<PackageRegistry>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_user_package(
        &self,
        user_id: Uuid,
        package_type: PackageType,
        package_id: &str,
    ) -> BizResult<Option<PackageRegistry>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(package_registry::Column::UserId.eq(user_id))
                    .filter(package_registry::Column::PackageType.eq(package_type.as_str()))
                    .filter(package_registry::Column::PackageId.eq(package_id)),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: PackageRegistryFilter) -> BizResult<Vec<PackageRegistry>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: PackageRegistryFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdatePackageRegistry) -> BizResult<PackageRegistry> {
        let model = package_registry::ActiveModel {
            id: Unchanged(input.id),
            package_name: Set(input.package_name),
            version: Set(input.version),
            status: Set(input.status.as_str().to_string()),
            manifest_path: Set(input.manifest_path),
            dependencies: Set(input.dependencies),
            compatibility: Set(input.compatibility),
            metadata: Set(input.metadata),
            updated_by: Set(input.updated_by),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: PackageRegistryFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<package_registry::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(package_registry::Column::UserId.eq(filter.user_id));

        if let Some(package_type) = filter.package_type {
            query = query.filter(package_registry::Column::PackageType.eq(package_type.as_str()));
        }

        if let Some(status) = filter.status {
            query = query.filter(package_registry::Column::Status.eq(status.as_str()));
        } else if !filter.include_disabled {
            query =
                query.filter(package_registry::Column::Status.ne(PackageStatus::Disabled.as_str()));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(package_registry::Column::PackageId.contains(keyword.clone()))
                .add(package_registry::Column::PackageName.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query
            .order_by_asc(package_registry::Column::PackageType)
            .order_by_asc(package_registry::Column::PackageId);

        if include_pagination {
            if let Some(offset) = filter.offset {
                query = query.offset(offset);
            }

            if let Some(limit) = filter.limit {
                query = query.limit(limit);
            }
        }

        query
    }

    fn from_model(model: package_registry::Model) -> BizResult<PackageRegistry> {
        Ok(PackageRegistry {
            id: model.id,
            user_id: model.user_id,
            package_type: PackageType::try_from(model.package_type.as_str())?,
            package_id: model.package_id,
            package_name: model.package_name,
            version: model.version,
            status: PackageStatus::try_from(model.status.as_str())?,
            manifest_path: model.manifest_path,
            dependencies: model.dependencies,
            compatibility: model.compatibility,
            metadata: model.metadata,
            updated_by: model.updated_by,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
