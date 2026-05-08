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
    entity::campaigns,
    table::campaigns::dto::{
        Campaign, CampaignFilter, CampaignStatus, CreateCampaign, UpdateCampaign,
    },
};

db_core::impl_repository!(CampaignRepo, campaigns::Entity, campaigns::Model);

pub struct CampaignService {
    repo: CampaignRepo,
}

impl CampaignService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: CampaignRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateCampaign) -> BizResult<Campaign> {
        let model = campaigns::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            product_id: Set(input.product_id),
            name: Set(input.name),
            campaign_type: Set(input.campaign_type),
            description: Set(input.description),
            budget: Set(input.budget),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            output: Set(input.output),
            action_metadata: Set(input.action_metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<Campaign>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn list(&self, filter: CampaignFilter) -> BizResult<Vec<Campaign>> {
        let mut query = self
            .repo
            .query()
            .filter(campaigns::Column::UserId.eq(filter.user_id));

        if let Some(product_id) = filter.product_id {
            query = query.filter(campaigns::Column::ProductId.eq(product_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(campaigns::Column::Status.eq(status.as_str()));
        }

        if let Some(campaign_type) = filter.campaign_type {
            query = query.filter(campaigns::Column::CampaignType.eq(campaign_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(campaigns::Column::Name.contains(keyword.clone()))
                .add(campaigns::Column::CampaignType.contains(keyword.clone()))
                .add(campaigns::Column::Description.contains(keyword));
            query = query.filter(keyword_filter);
        }

        query = query.order_by_desc(campaigns::Column::UpdatedAt);

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

    pub async fn count(&self, filter: CampaignFilter) -> BizResult<u64> {
        let mut query = self
            .repo
            .query()
            .filter(campaigns::Column::UserId.eq(filter.user_id));

        if let Some(product_id) = filter.product_id {
            query = query.filter(campaigns::Column::ProductId.eq(product_id));
        }

        if let Some(status) = filter.status {
            query = query.filter(campaigns::Column::Status.eq(status.as_str()));
        }

        if let Some(campaign_type) = filter.campaign_type {
            query = query.filter(campaigns::Column::CampaignType.eq(campaign_type));
        }

        if let Some(keyword) = filter.keyword {
            let keyword_filter = Condition::any()
                .add(campaigns::Column::Name.contains(keyword.clone()))
                .add(campaigns::Column::CampaignType.contains(keyword.clone()))
                .add(campaigns::Column::Description.contains(keyword));
            query = query.filter(keyword_filter);
        }

        Ok(query.count(self.repo.db()).await.map_err(Error::from)?)
    }

    pub async fn update(&self, input: UpdateCampaign) -> BizResult<Campaign> {
        let model = campaigns::ActiveModel {
            id: Unchanged(input.id),
            product_id: Set(input.product_id),
            name: Set(input.name),
            campaign_type: Set(input.campaign_type),
            description: Set(input.description),
            budget: Set(input.budget),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            status: Set(input.status.as_str().to_string()),
            metadata: Set(input.metadata),
            output: Set(input.output),
            action_metadata: Set(input.action_metadata),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: campaigns::Model) -> BizResult<Campaign> {
        Ok(Campaign {
            id: model.id,
            user_id: model.user_id,
            product_id: model.product_id,
            name: model.name,
            campaign_type: model.campaign_type,
            description: model.description,
            budget: model.budget,
            start_date: model.start_date,
            end_date: model.end_date,
            status: CampaignStatus::try_from(model.status)?,
            metadata: model.metadata,
            output: model.output,
            action_metadata: model.action_metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
