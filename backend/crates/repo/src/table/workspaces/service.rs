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
    entity::workspaces,
    table::workspaces::dto::{
        CreateWorkspace, UpdateWorkspace, Workspace, WorkspaceFilter, WorkspaceStatus,
    },
};

db_core::impl_repository!(WorkspaceRepo, workspaces::Entity, workspaces::Model);

pub struct WorkspaceService {
    repo: WorkspaceRepo,
}

impl WorkspaceService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: WorkspaceRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateWorkspace) -> BizResult<Workspace> {
        let model = workspaces::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            campaign_id: Set(input.campaign_id),
            title: Set(input.title),
            workspace_type: Set(input.workspace_type),
            status: Set(input.status.as_str().to_string()),
            config: Set(input.config),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<Workspace>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: WorkspaceFilter) -> BizResult<Vec<Workspace>> {
        let query = self.apply_filter(filter, true);
        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn count(&self, filter: WorkspaceFilter) -> BizResult<u64> {
        Ok(self
            .apply_filter(filter, false)
            .count(self.repo.db())
            .await
            .map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateWorkspace) -> BizResult<Workspace> {
        let model = workspaces::ActiveModel {
            id: Unchanged(input.id),
            product_id: Set(input.product_id),
            campaign_id: Set(input.campaign_id),
            title: Set(input.title),
            workspace_type: Set(input.workspace_type),
            status: Set(input.status.as_str().to_string()),
            config: Set(input.config),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn apply_filter(
        &self,
        filter: WorkspaceFilter,
        include_pagination: bool,
    ) -> sea_orm::Select<workspaces::Entity> {
        let mut query = self
            .repo
            .query()
            .filter(workspaces::Column::UserId.eq(filter.user_id));

        if let Some(status) = filter.status {
            query = query.filter(workspaces::Column::Status.eq(status.as_str()));
        }

        if let Some(product_id) = filter.product_id {
            query = query.filter(workspaces::Column::ProductId.eq(product_id));
        }

        if let Some(campaign_id) = filter.campaign_id {
            query = query.filter(workspaces::Column::CampaignId.eq(campaign_id));
        }

        if let Some(workspace_type) = filter.workspace_type {
            query = query.filter(workspaces::Column::WorkspaceType.eq(workspace_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(workspaces::Column::Title.contains(keyword.clone()))
                .add(workspaces::Column::WorkspaceType.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(workspaces::Column::UpdatedAt);

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

    fn from_model(model: workspaces::Model) -> BizResult<Workspace> {
        Ok(Workspace {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            campaign_id: model.campaign_id,
            title: model.title,
            workspace_type: model.workspace_type,
            status: WorkspaceStatus::try_from(model.status.as_str())?,
            config: model.config,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
