use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    entity::alerts,
    table::alerts::dto::{Alert, AlertFilter, AlertStatus, CreateAlert, UpdateAlert},
};

db_core::impl_repository!(AlertRepo, alerts::Entity, alerts::Model);

pub struct AlertService {
    repo: AlertRepo,
}

impl AlertService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: AlertRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateAlert) -> BizResult<Alert> {
        let model = alerts::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            workspace_id: Set(input.workspace_id),
            run_id: Set(input.run_id),
            alert_type: Set(input.alert_type),
            severity: Set(input.severity),
            message: Set(input.message),
            role: Set(input.role),
            status: Set(input.status.as_str().to_string()),
            dedup_key: Set(input.dedup_key),
            snoozed_until: Set(input.snoozed_until),
            payload: Set(input.payload),
            acted_at: Set(input.acted_at),
            dismissed_at: Set(input.dismissed_at),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<Alert>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_active_by_dedup(
        &self,
        user_id: Uuid,
        dedup_key: &str,
    ) -> BizResult<Option<Alert>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(alerts::Column::UserId.eq(user_id))
                    .filter(alerts::Column::DedupKey.eq(dedup_key))
                    .filter(alerts::Column::Status.eq(AlertStatus::Active.as_str())),
            )
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: AlertFilter) -> BizResult<Vec<Alert>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: AlertFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateAlert) -> BizResult<Alert> {
        let model = alerts::ActiveModel {
            id: Unchanged(input.id),
            status: Set(input.status.as_str().to_string()),
            snoozed_until: Set(input.snoozed_until),
            acted_at: Set(input.acted_at),
            dismissed_at: Set(input.dismissed_at),
            payload: Set(input.payload),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    pub async fn count_group_by_severity(&self, user_id: Uuid) -> BizResult<Vec<(String, i64)>> {
        Ok(alerts::Entity::find()
            .select_only()
            .column(alerts::Column::Severity)
            .column_as(alerts::Column::Id.count(), "total")
            .filter(alerts::Column::UserId.eq(user_id))
            .filter(alerts::Column::Status.eq(AlertStatus::Active.as_str()))
            .group_by(alerts::Column::Severity)
            .into_tuple::<(String, i64)>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn count_group_by_type(&self, user_id: Uuid) -> BizResult<Vec<(String, i64)>> {
        Ok(alerts::Entity::find()
            .select_only()
            .column(alerts::Column::AlertType)
            .column_as(alerts::Column::Id.count(), "total")
            .filter(alerts::Column::UserId.eq(user_id))
            .filter(alerts::Column::Status.eq(AlertStatus::Active.as_str()))
            .group_by(alerts::Column::AlertType)
            .into_tuple::<(String, i64)>()
            .all(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    fn apply_filter(
        &self,
        filter: AlertFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<alerts::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(alerts::Column::UserId.eq(filter.user_id));

        if let Some(workspace_id) = filter.workspace_id {
            query = query.filter(alerts::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(run_id) = filter.run_id {
            query = query.filter(alerts::Column::RunId.eq(run_id));
        }

        if let Some(alert_type) = filter.alert_type {
            query = query.filter(alerts::Column::AlertType.eq(alert_type));
        }

        if let Some(severity) = filter.severity {
            query = query.filter(alerts::Column::Severity.eq(severity));
        }

        if let Some(status) = filter.status {
            query = query.filter(alerts::Column::Status.eq(status.as_str()));
        }

        if !filter.include_snoozed {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(alerts::Column::SnoozedUntil.is_null())
                    .add(alerts::Column::SnoozedUntil.lte(OffsetDateTime::now_utc())),
            );
        }

        query = query.order_by_desc(alerts::Column::CreatedAt);

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

    fn from_model(model: alerts::Model) -> BizResult<Alert> {
        Ok(Alert {
            id: model.id,
            user_id: model.user_id,
            workspace_id: model.workspace_id,
            run_id: model.run_id,
            alert_type: model.alert_type,
            severity: model.severity,
            message: model.message,
            role: model.role,
            status: AlertStatus::try_from(model.status.as_str())?,
            dedup_key: model.dedup_key,
            snoozed_until: model.snoozed_until,
            payload: model.payload,
            acted_at: model.acted_at,
            dismissed_at: model.dismissed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
