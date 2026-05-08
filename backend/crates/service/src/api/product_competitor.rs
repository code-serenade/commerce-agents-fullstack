use std::collections::HashMap;

use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    competitor_changes::{
        CompetitorChange, CompetitorChangeFilter, CompetitorChangeService, CreateCompetitorChange,
    },
    product_competitors::{
        CreateProductCompetitor, ProductCompetitor, ProductCompetitorFilter,
        ProductCompetitorService, ProductCompetitorStatus, UpdateProductCompetitor,
    },
    products::{Product, ProductService, ProductStatus},
};
use serde_json::json;
use uuid::Uuid;

use crate::dto::product_competitor::{
    CompetitorChangeResponse, CreateCompetitorChangeRequest, CreateProductCompetitorRequest,
    DeleteProductCompetitorRequest, GetProductCompetitorRequest, ListCompetitorChangesRequest,
    ListProductCompetitorsRequest, ProductCompetitorResponse, UpdateProductCompetitorRequest,
};

const DEFAULT_COMPETITOR_LIST_LIMIT: u64 = 200;
const DEFAULT_CHANGE_LIST_LIMIT: u64 = 100;
const DEFAULT_CHANGE_SOURCE_TYPE: &str = "manual";

pub struct ProductCompetitorApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    competitor_svc: ProductCompetitorService,
    change_svc: CompetitorChangeService,
}

impl ProductCompetitorApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            competitor_svc: ProductCompetitorService::new(db.clone()),
            change_svc: CompetitorChangeService::new(db),
        }
    }

    pub async fn list_competitors(
        &self,
        req: ListProductCompetitorsRequest,
    ) -> BizResult<Vec<ProductCompetitorResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let status = parse_optional_status(req.status)?;

        self.competitor_svc
            .list(ProductCompetitorFilter {
                user_id,
                product_id,
                status: Some(status.unwrap_or(ProductCompetitorStatus::Active)),
                platform: normalize_optional(req.platform),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_COMPETITOR_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_competitor)
            .collect()
    }

    pub async fn create_competitor(
        &self,
        req: CreateProductCompetitorRequest,
    ) -> BizResult<ProductCompetitorResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let name = normalize_required(req.name, "name")?;

        let competitor = self
            .competitor_svc
            .create(CreateProductCompetitor {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                name,
                platform: normalize_optional(req.platform),
                url: normalize_optional(req.url),
                price: req.price,
                rating: req.rating,
                monthly_sales: normalize_optional(req.monthly_sales),
                notes: normalize_optional(req.notes),
                search_results: req.search_results.unwrap_or_else(|| json!({})),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
                last_searched_at: req.last_searched_at,
            })
            .await?;

        Self::map_competitor(competitor)
    }

    pub async fn get_competitor(
        &self,
        req: GetProductCompetitorRequest,
    ) -> BizResult<ProductCompetitorResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let competitor_id = parse_uuid(&req.competitor_id, "competitor_id")?;
        let competitor = self
            .ensure_active_competitor_owner(user_id, product_id, competitor_id)
            .await?;

        Self::map_competitor(competitor)
    }

    pub async fn update_competitor(
        &self,
        req: UpdateProductCompetitorRequest,
    ) -> BizResult<ProductCompetitorResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let competitor_id = parse_uuid(&req.competitor_id, "competitor_id")?;
        let current = self
            .ensure_active_competitor_owner(user_id, product_id, competitor_id)
            .await?;

        let competitor = self
            .competitor_svc
            .update(UpdateProductCompetitor {
                id: current.id,
                name: req
                    .name
                    .map(|value| normalize_required(value, "name"))
                    .transpose()?
                    .unwrap_or(current.name),
                platform: req
                    .platform
                    .map(normalize_string)
                    .unwrap_or(current.platform),
                url: req.url.map(normalize_string).unwrap_or(current.url),
                price: req.price.or(current.price),
                rating: req.rating.or(current.rating),
                monthly_sales: req
                    .monthly_sales
                    .map(normalize_string)
                    .unwrap_or(current.monthly_sales),
                notes: req.notes.map(normalize_string).unwrap_or(current.notes),
                search_results: req.search_results.unwrap_or(current.search_results),
                status: current.status,
                metadata: req.metadata.unwrap_or(current.metadata),
                last_searched_at: req.last_searched_at.or(current.last_searched_at),
            })
            .await?;

        Self::map_competitor(competitor)
    }

    pub async fn delete_competitor(&self, req: DeleteProductCompetitorRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let competitor_id = parse_uuid(&req.competitor_id, "competitor_id")?;
        let current = self
            .ensure_active_competitor_owner(user_id, product_id, competitor_id)
            .await?;

        self.competitor_svc
            .update(UpdateProductCompetitor {
                id: current.id,
                name: current.name,
                platform: current.platform,
                url: current.url,
                price: current.price,
                rating: current.rating,
                monthly_sales: current.monthly_sales,
                notes: current.notes,
                search_results: current.search_results,
                status: ProductCompetitorStatus::Archived,
                metadata: current.metadata,
                last_searched_at: current.last_searched_at,
            })
            .await?;

        Ok(())
    }

    pub async fn list_changes(
        &self,
        req: ListCompetitorChangesRequest,
    ) -> BizResult<Vec<CompetitorChangeResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let competitor_id = req
            .competitor_id
            .map(|value| parse_uuid(&value, "competitor_id"))
            .transpose()?;

        if let Some(competitor_id) = competitor_id {
            self.ensure_competitor_owner(user_id, product_id, competitor_id)
                .await?;
        }

        let changes = self
            .change_svc
            .list(CompetitorChangeFilter {
                user_id,
                product_id,
                competitor_id,
                change_type: normalize_optional(req.change_type),
                limit: Some(req.limit.unwrap_or(DEFAULT_CHANGE_LIST_LIMIT)),
            })
            .await?;
        let competitors = self.list_competitor_lookup(user_id, product_id).await?;

        changes
            .into_iter()
            .map(|change| Self::map_change(change, &competitors))
            .collect()
    }

    pub async fn create_change(
        &self,
        req: CreateCompetitorChangeRequest,
    ) -> BizResult<CompetitorChangeResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        self.ensure_active_product_owner(user_id, product_id)
            .await?;
        let competitor_id = parse_uuid(&req.competitor_id, "competitor_id")?;
        let competitor = self
            .ensure_active_competitor_owner(user_id, product_id, competitor_id)
            .await?;
        let change_type = normalize_required(req.change_type, "change_type")?;

        let change = self
            .change_svc
            .create(CreateCompetitorChange {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                competitor_id,
                change_type,
                old_value: normalize_optional(req.old_value),
                new_value: normalize_optional(req.new_value),
                summary: normalize_optional(req.summary),
                source_type: normalize_optional(req.source_type)
                    .unwrap_or_else(|| DEFAULT_CHANGE_SOURCE_TYPE.to_string()),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
                detected_at: req.detected_at,
            })
            .await?;

        let competitors = HashMap::from([(competitor.id, competitor)]);
        Self::map_change(change, &competitors)
    }

    async fn list_competitor_lookup(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> BizResult<HashMap<Uuid, ProductCompetitor>> {
        let competitors = self
            .competitor_svc
            .list(ProductCompetitorFilter {
                user_id,
                product_id,
                status: None,
                platform: None,
                keyword: None,
                limit: Some(DEFAULT_COMPETITOR_LIST_LIMIT),
            })
            .await?;

        Ok(competitors
            .into_iter()
            .map(|competitor| (competitor.id, competitor))
            .collect())
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

    async fn ensure_competitor_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        competitor_id: Uuid,
    ) -> BizResult<ProductCompetitor> {
        let competitor = self
            .competitor_svc
            .get_by_id(competitor_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_COMPETITOR_NOT_FOUND,
                    "competitor not found".to_string(),
                )
            })?;
        if competitor.user_id != user_id || competitor.product_id != product_id {
            return Err(BizError::new(
                app_error::PRODUCT_COMPETITOR_FORBIDDEN,
                "competitor does not belong to current product".to_string(),
            ));
        }
        Ok(competitor)
    }

    async fn ensure_active_competitor_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        competitor_id: Uuid,
    ) -> BizResult<ProductCompetitor> {
        let competitor = self
            .ensure_competitor_owner(user_id, product_id, competitor_id)
            .await?;
        if competitor.status == ProductCompetitorStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_COMPETITOR_NOT_FOUND,
                "competitor not found".to_string(),
            ));
        }
        Ok(competitor)
    }

    fn map_competitor(item: ProductCompetitor) -> BizResult<ProductCompetitorResponse> {
        Ok(ProductCompetitorResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.to_string(),
            name: item.name,
            platform: item.platform,
            url: item.url,
            price: item.price,
            rating: item.rating,
            monthly_sales: item.monthly_sales,
            notes: item.notes,
            search_results: item.search_results,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            last_searched_at: item.last_searched_at,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }

    fn map_change(
        item: CompetitorChange,
        competitors: &HashMap<Uuid, ProductCompetitor>,
    ) -> BizResult<CompetitorChangeResponse> {
        let competitor = competitors.get(&item.competitor_id);
        Ok(CompetitorChangeResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.to_string(),
            competitor_id: item.competitor_id.to_string(),
            competitor_name: competitor.map(|item| item.name.clone()),
            competitor_platform: competitor.and_then(|item| item.platform.clone()),
            change_type: item.change_type,
            old_value: item.old_value,
            new_value: item.new_value,
            summary: item.summary,
            source_type: item.source_type,
            metadata: item.metadata,
            detected_at: item.detected_at,
            created_at: item.created_at,
        })
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PRODUCT_COMPETITOR_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductCompetitorStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "active" => Ok(ProductCompetitorStatus::Active),
            "archived" => Ok(ProductCompetitorStatus::Archived),
            _ => Err(BizError::new(
                app_error::PRODUCT_COMPETITOR_REQUEST_INVALID,
                format!("invalid product competitor status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_COMPETITOR_REQUEST_INVALID,
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
