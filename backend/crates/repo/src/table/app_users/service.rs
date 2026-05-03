use db_core::{
    DbContext, OrderBy, PaginatedResponse, PaginationParams, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, PaginatorTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::app_users,
    table::app_users::dto::{
        AppUser, AppUserFilter, AppUserMetrics, AppUserSortBy, AppUserStatus, CreateAppUser,
        SortOrder, UpdateAppUser,
    },
};

db_core::impl_repository!(AppUserRepo, app_users::Entity, app_users::Model);

pub struct AppUserService {
    repo: AppUserRepo,
}

impl AppUserService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AppUserRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAppUser) -> BizResult<AppUser> {
        let model = app_users::ActiveModel {
            user_id: Set(input.user_id),
            display_id: Set(input.display_id),
            display_name: Set(input.display_name),
            remark: Set(input.remark),
            status: Set(input.status.as_str().to_owned()),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list_all(&self) -> BizResult<Vec<AppUser>> {
        let query = self.repo.query().order_by_asc(app_users::Column::UserId);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn metrics(&self) -> BizResult<AppUserMetrics> {
        let total = self
            .repo
            .query()
            .count(self.repo.db())
            .await
            .map_err(Error::from)?;
        let enabled = self
            .repo
            .query()
            .filter(app_users::Column::Status.eq(AppUserStatus::Enabled.as_str()))
            .count(self.repo.db())
            .await
            .map_err(Error::from)?;
        let disabled = self
            .repo
            .query()
            .filter(app_users::Column::Status.eq(AppUserStatus::Disabled.as_str()))
            .count(self.repo.db())
            .await
            .map_err(Error::from)?;

        Ok(AppUserMetrics {
            total,
            enabled,
            disabled,
        })
    }

    pub async fn list_paginated(
        &self,
        filter: AppUserFilter,
        pagination: PaginationParams,
    ) -> BizResult<PaginatedResponse<AppUser>> {
        let page = pagination.validate();
        let order_by = Self::build_order_by(&filter);
        let filter = Self::build_filter(filter);
        let response = self
            .repo
            .find_paginated(filter, &page, Some(&order_by))
            .await?;

        let items = response
            .items
            .into_iter()
            .map(Self::from_model)
            .collect::<BizResult<Vec<_>>>()?;

        Ok(PaginatedResponse {
            items,
            page: response.page,
            page_size: response.page_size,
            total: response.total,
            total_pages: response.total_pages,
            has_next: response.has_next,
            has_prev: response.has_prev,
        })
    }

    pub async fn get_by_user_id(&self, user_id: Uuid) -> BizResult<Option<AppUser>> {
        self.repo
            .find_by_id(user_id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn update(&self, input: UpdateAppUser) -> BizResult<AppUser> {
        let model = app_users::ActiveModel {
            user_id: Unchanged(input.user_id),
            remark: Set(input.remark),
            status: Set(input.status.as_str().to_owned()),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    pub async fn delete_by_user_id(&self, user_id: Uuid) -> BizResult<u64> {
        Ok(self.repo.delete_by_id(user_id).await?.rows_affected)
    }

    fn from_model(model: app_users::Model) -> BizResult<AppUser> {
        Ok(AppUser {
            user_id: model.user_id,
            display_id: model.display_id,
            display_name: model.display_name,
            remark: model.remark,
            status: AppUserStatus::try_from(model.status)?,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    fn build_filter(filter: AppUserFilter) -> Option<Condition> {
        let mut condition = Condition::all();
        let mut has_filter = false;

        if let Some(keyword) = filter.keyword {
            let mut keyword_condition = Condition::any()
                .add(app_users::Column::DisplayId.contains(keyword.clone()))
                .add(app_users::Column::DisplayName.contains(keyword.clone()))
                .add(app_users::Column::Remark.contains(keyword));

            if let Some(user_id) = filter.keyword_user_id {
                keyword_condition = keyword_condition.add(app_users::Column::UserId.eq(user_id));
            }

            condition = condition.add(keyword_condition);
            has_filter = true;
        }

        if let Some(status) = filter.status {
            condition = condition.add(app_users::Column::Status.eq(status.as_str()));
            has_filter = true;
        }

        if let Some(created_at_from) = filter.created_at_from {
            condition = condition.add(app_users::Column::CreatedAt.gte(created_at_from));
            has_filter = true;
        }

        if let Some(created_at_to) = filter.created_at_to {
            condition = condition.add(app_users::Column::CreatedAt.lte(created_at_to));
            has_filter = true;
        }

        if let Some(updated_at_from) = filter.updated_at_from {
            condition = condition.add(app_users::Column::UpdatedAt.gte(updated_at_from));
            has_filter = true;
        }

        if let Some(updated_at_to) = filter.updated_at_to {
            condition = condition.add(app_users::Column::UpdatedAt.lte(updated_at_to));
            has_filter = true;
        }

        has_filter.then_some(condition)
    }

    fn build_order_by(filter: &AppUserFilter) -> OrderBy<app_users::Entity> {
        let Some(sort_by) = filter.sort_by else {
            return OrderBy::desc(app_users::Column::UpdatedAt);
        };

        let column = match sort_by {
            AppUserSortBy::CreatedAt => app_users::Column::CreatedAt,
            AppUserSortBy::UpdatedAt => app_users::Column::UpdatedAt,
        };

        match filter.sort_order {
            Some(SortOrder::Desc) => OrderBy::desc(column),
            Some(SortOrder::Asc) => OrderBy::asc(column),
            None => OrderBy::desc(column),
        }
    }
}
