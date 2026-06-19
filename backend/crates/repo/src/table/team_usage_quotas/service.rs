use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    entity::team_usage_quotas,
    table::team_usage_quotas::dto::{
        CreateTeamUsageQuota, TeamUsageQuota, TeamUsageQuotaStatus, UpdateTeamUsageQuota,
    },
};

db_core::impl_repository!(
    TeamUsageQuotaRepo,
    team_usage_quotas::Entity,
    team_usage_quotas::Model
);

pub struct TeamUsageQuotaService {
    repo: TeamUsageQuotaRepo,
}

impl TeamUsageQuotaService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: TeamUsageQuotaRepo::new(db),
        }
    }

    pub async fn create(&self, input: CreateTeamUsageQuota) -> BizResult<TeamUsageQuota> {
        let model = team_usage_quotas::ActiveModel {
            id: Set(input.id),
            school_name: Set(input.school_name),
            team_name: Set(input.team_name),
            question_limit: Set(input.question_limit),
            token_limit: Set(input.token_limit),
            status: Set(input.status.as_str().to_owned()),
            remark: Set(input.remark),
            ..Default::default()
        };

        Self::from_model(self.repo.insert(model).await?)
    }

    pub async fn list_all(&self) -> BizResult<Vec<TeamUsageQuota>> {
        let query = self
            .repo
            .query()
            .order_by_asc(team_usage_quotas::Column::SchoolName)
            .order_by_asc(team_usage_quotas::Column::TeamName);

        self.repo
            .select_all(query)
            .await?
            .into_iter()
            .map(Self::from_model)
            .collect()
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<TeamUsageQuota>> {
        self.repo
            .find_by_id(id)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn get_by_team(
        &self,
        school_name: &str,
        team_name: &str,
    ) -> BizResult<Option<TeamUsageQuota>> {
        let query = self
            .repo
            .query()
            .filter(team_usage_quotas::Column::SchoolName.eq(school_name.to_owned()))
            .filter(team_usage_quotas::Column::TeamName.eq(team_name.to_owned()));

        self.repo
            .select_one(query)
            .await?
            .map(Self::from_model)
            .transpose()
    }

    pub async fn update(&self, input: UpdateTeamUsageQuota) -> BizResult<TeamUsageQuota> {
        let model = team_usage_quotas::ActiveModel {
            id: Unchanged(input.id),
            question_limit: Set(input.question_limit),
            token_limit: Set(input.token_limit),
            status: Set(input.status.as_str().to_owned()),
            remark: Set(input.remark),
            ..Default::default()
        };

        Self::from_model(self.repo.update(model).await?)
    }

    fn from_model(model: team_usage_quotas::Model) -> BizResult<TeamUsageQuota> {
        Ok(TeamUsageQuota {
            id: model.id,
            school_name: model.school_name,
            team_name: model.team_name,
            question_limit: model.question_limit,
            token_limit: model.token_limit,
            status: TeamUsageQuotaStatus::try_from(model.status)?,
            remark: model.remark,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
