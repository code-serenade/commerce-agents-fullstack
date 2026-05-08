use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    campaigns::{
        Campaign, CampaignFilter, CampaignService, CampaignStatus, CreateCampaign, UpdateCampaign,
    },
    products::{Product, ProductService, ProductStatus},
};
use serde_json::json;
use uuid::Uuid;

use crate::dto::campaign::{
    CampaignListResponse, CampaignResponse, CreateCampaignRequest, DeleteCampaignRequest,
    GetCampaignRequest, ListCampaignsRequest, UpdateCampaignRequest,
};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct CampaignApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    campaign_svc: CampaignService,
}

impl CampaignApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            campaign_svc: CampaignService::new(db),
        }
    }

    pub async fn list_campaigns(
        &self,
        req: ListCampaignsRequest,
    ) -> BizResult<CampaignListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let status = parse_optional_status(req.status)?;
        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }

        let page = req.page.unwrap_or(DEFAULT_PAGE).max(1);
        let page_size = req
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        let campaign_type = normalize_optional(req.campaign_type);
        let keyword = normalize_optional(req.keyword);
        let filter = CampaignFilter {
            user_id,
            product_id,
            status,
            campaign_type,
            keyword,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = CampaignFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };

        let total = self.campaign_svc.count(count_filter).await?;
        let campaigns = self
            .campaign_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_campaign)
            .collect::<BizResult<Vec<_>>>()?;

        Ok(CampaignListResponse {
            campaigns,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_campaign(&self, req: CreateCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let name = normalize_required(req.name, "name")?;
        let status = parse_optional_status(req.status)?.unwrap_or(CampaignStatus::Draft);

        let campaign = self
            .campaign_svc
            .create(CreateCampaign {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                name,
                campaign_type: normalize_optional(req.campaign_type),
                description: normalize_optional(req.description),
                budget: req.budget.unwrap_or_default(),
                start_date: normalize_optional(req.start_date),
                end_date: normalize_optional(req.end_date),
                status,
                metadata: req.metadata.unwrap_or_else(|| json!({})),
                output: req.output.unwrap_or_else(|| json!({})),
                action_metadata: req.action_metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_campaign(campaign)
    }

    pub async fn get_campaign(&self, req: GetCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let campaign = self.ensure_campaign_owner(user_id, campaign_id).await?;
        Self::map_campaign(campaign)
    }

    pub async fn update_campaign(&self, req: UpdateCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let current = self.ensure_campaign_owner(user_id, campaign_id).await?;

        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?
            .or(current.product_id);
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }

        let status = parse_optional_status(req.status)?.unwrap_or(current.status);
        let campaign = self
            .campaign_svc
            .update(UpdateCampaign {
                id: current.id,
                product_id,
                name: req
                    .name
                    .map(|value| normalize_required(value, "name"))
                    .transpose()?
                    .unwrap_or(current.name),
                campaign_type: req
                    .campaign_type
                    .map(normalize_string)
                    .unwrap_or(current.campaign_type),
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                budget: req.budget.unwrap_or(current.budget),
                start_date: req
                    .start_date
                    .map(normalize_string)
                    .unwrap_or(current.start_date),
                end_date: req
                    .end_date
                    .map(normalize_string)
                    .unwrap_or(current.end_date),
                status,
                metadata: req.metadata.unwrap_or(current.metadata),
                output: req.output.unwrap_or(current.output),
                action_metadata: req.action_metadata.unwrap_or(current.action_metadata),
            })
            .await?;

        Self::map_campaign(campaign)
    }

    pub async fn delete_campaign(&self, req: DeleteCampaignRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let current = self.ensure_campaign_owner(user_id, campaign_id).await?;

        self.campaign_svc
            .update(UpdateCampaign {
                id: current.id,
                product_id: current.product_id,
                name: current.name,
                campaign_type: current.campaign_type,
                description: current.description,
                budget: current.budget,
                start_date: current.start_date,
                end_date: current.end_date,
                status: CampaignStatus::Archived,
                metadata: current.metadata,
                output: current.output,
                action_metadata: current.action_metadata,
            })
            .await?;

        Ok(())
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_product_owner(&self, user_id: Uuid, product_id: Uuid) -> BizResult<Product> {
        let product = self
            .product_svc
            .get_by_id(product_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_NOT_FOUND,
                    "product not found".to_string(),
                )
            })?;
        if product.user_id != user_id {
            return Err(BizError::new(
                app_error::PRODUCT_FORBIDDEN,
                "product does not belong to current user".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_active_product_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> BizResult<Product> {
        let product = self.ensure_product_owner(user_id, product_id).await?;
        if product.status == ProductStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_NOT_FOUND,
                "product not found".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_campaign_owner(&self, user_id: Uuid, campaign_id: Uuid) -> BizResult<Campaign> {
        let campaign = self
            .campaign_svc
            .get_by_id(campaign_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::CAMPAIGN_NOT_FOUND,
                    "campaign not found".to_string(),
                )
            })?;
        if campaign.user_id != user_id {
            return Err(BizError::new(
                app_error::CAMPAIGN_FORBIDDEN,
                "campaign does not belong to current user".to_string(),
            ));
        }
        Ok(campaign)
    }

    fn map_campaign(item: Campaign) -> BizResult<CampaignResponse> {
        Ok(CampaignResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.map(|id| id.to_string()),
            name: item.name,
            campaign_type: item.campaign_type,
            description: item.description,
            budget: item.budget,
            start_date: item.start_date,
            end_date: item.end_date,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            output: item.output,
            action_metadata: item.action_metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::CAMPAIGN_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<CampaignStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "draft" => Ok(CampaignStatus::Draft),
            "active" => Ok(CampaignStatus::Active),
            "paused" => Ok(CampaignStatus::Paused),
            "ended" => Ok(CampaignStatus::Ended),
            "archived" => Ok(CampaignStatus::Archived),
            _ => Err(BizError::new(
                app_error::CAMPAIGN_REQUEST_INVALID,
                format!("invalid campaign status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::CAMPAIGN_REQUEST_INVALID,
            format!("{label} cannot be empty"),
        ));
    }
    Ok(value)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(normalize_string)
}

fn normalize_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}
