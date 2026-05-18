use std::collections::{BTreeMap, HashMap};

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
    DeleteProductCompetitorRequest, GetProductCompetitorAnalysisRequest,
    GetProductCompetitorRequest, ListCompetitorChangesRequest, ListProductCompetitorsRequest,
    ProductCompetitorAnalysisCounts, ProductCompetitorAnalysisProductResponse,
    ProductCompetitorAnalysisRefResponse, ProductCompetitorAnalysisResponse,
    ProductCompetitorPlatformAnalysisResponse, ProductCompetitorPriceAnalysisResponse,
    ProductCompetitorRatingAnalysisResponse, ProductCompetitorResponse,
    UpdateProductCompetitorRequest,
};

const DEFAULT_COMPETITOR_LIST_LIMIT: u64 = 200;
const DEFAULT_CHANGE_LIST_LIMIT: u64 = 100;
const DEFAULT_CHANGE_SOURCE_TYPE: &str = "manual";
const TOP_COMPETITOR_LIMIT: usize = 10;

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

    pub async fn competitor_analysis(
        &self,
        req: GetProductCompetitorAnalysisRequest,
    ) -> BizResult<ProductCompetitorAnalysisResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let product = self
            .ensure_active_product_owner(user_id, product_id)
            .await?;
        let status = parse_optional_status(req.status)?;
        let competitors = self
            .competitor_svc
            .list(ProductCompetitorFilter {
                user_id,
                product_id,
                status: Some(status.unwrap_or(ProductCompetitorStatus::Active)),
                platform: normalize_optional(req.platform),
                keyword: None,
                limit: Some(DEFAULT_COMPETITOR_LIST_LIMIT),
            })
            .await?;
        let changes = self
            .change_svc
            .list(CompetitorChangeFilter {
                user_id,
                product_id,
                competitor_id: None,
                change_type: None,
                limit: Some(req.change_limit.unwrap_or(DEFAULT_CHANGE_LIST_LIMIT)),
            })
            .await?;
        let competitors_lookup = self.list_competitor_lookup(user_id, product_id).await?;

        Self::build_competitor_analysis(product, competitors, changes, &competitors_lookup)
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

    fn build_competitor_analysis(
        product: Product,
        competitors: Vec<ProductCompetitor>,
        changes: Vec<CompetitorChange>,
        competitors_lookup: &HashMap<Uuid, ProductCompetitor>,
    ) -> BizResult<ProductCompetitorAnalysisResponse> {
        let product_price = parse_price(product.price.as_deref());
        let competitor_prices: Vec<f64> = competitors
            .iter()
            .filter_map(|item| item.price)
            .filter(|value| value.is_finite() && *value >= 0.0)
            .collect();
        let competitor_ratings: Vec<f64> = competitors
            .iter()
            .filter_map(|item| item.rating)
            .filter(|value| value.is_finite() && *value >= 0.0)
            .collect();
        let monthly_sales_competitors = competitors
            .iter()
            .filter(|item| {
                item.monthly_sales
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            })
            .count();
        let counts = ProductCompetitorAnalysisCounts {
            competitors: competitors.len(),
            priced_competitors: competitor_prices.len(),
            rated_competitors: competitor_ratings.len(),
            monthly_sales_competitors,
            recent_changes: changes.len(),
        };
        let price = build_price_analysis(product_price, &competitor_prices);
        let rating = build_rating_analysis(&competitor_ratings);
        let platforms = build_platform_analysis(&competitors);
        let readiness = analysis_readiness(&counts);
        let mut warnings = build_analysis_warnings(&counts);
        if product_price.is_none() {
            warnings.push("product price is missing or not numeric".to_string());
        }
        let signals = build_analysis_signals(&counts, &price, &changes);
        let summary = build_analysis_summary(&readiness, &counts, &price);
        let top_competitors = competitors
            .iter()
            .take(TOP_COMPETITOR_LIMIT)
            .cloned()
            .map(Self::map_competitor)
            .collect::<BizResult<Vec<_>>>()?;
        let recent_changes = changes
            .into_iter()
            .map(|change| Self::map_change(change, competitors_lookup))
            .collect::<BizResult<Vec<_>>>()?;
        let refs = build_analysis_refs(&top_competitors, &recent_changes);
        let product = ProductCompetitorAnalysisProductResponse {
            id: product.id.to_string(),
            name: product.name,
            platform: product.platform,
            category: product.category,
            brand: product.brand,
            price: product.price,
            numeric_price: product_price,
            currency: product.currency,
        };
        let agent_context = json!({
            "type": "product_competitor_analysis",
            "productId": product.id,
            "readiness": readiness,
            "summary": summary,
            "counts": counts,
            "price": price,
            "rating": rating,
            "platforms": platforms,
            "signals": signals,
            "refs": refs,
            "warnings": warnings,
        });

        Ok(ProductCompetitorAnalysisResponse {
            product,
            readiness,
            summary,
            counts,
            price,
            rating,
            platforms,
            top_competitors,
            recent_changes,
            refs,
            signals,
            warnings,
            agent_context,
        })
    }
}

#[derive(Debug, Default)]
struct PlatformAccumulator {
    competitors: usize,
    price_sum: f64,
    price_count: usize,
    rating_sum: f64,
    rating_count: usize,
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

fn parse_price(value: Option<&str>) -> Option<f64> {
    let normalized: String = value?
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .collect();
    normalized
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(round2)
}

fn build_price_analysis(
    product_price: Option<f64>,
    competitor_prices: &[f64],
) -> ProductCompetitorPriceAnalysisResponse {
    let min_competitor_price = competitor_prices
        .iter()
        .copied()
        .reduce(f64::min)
        .map(round2);
    let max_competitor_price = competitor_prices
        .iter()
        .copied()
        .reduce(f64::max)
        .map(round2);
    let avg_competitor_price = average(competitor_prices).map(round2);
    let (gap_to_avg, gap_percent_to_avg, price_position) =
        match (product_price, avg_competitor_price) {
            (Some(product_price), Some(avg_price)) if avg_price > 0.0 => {
                let gap = round2(product_price - avg_price);
                let gap_percent = round2(gap / avg_price * 100.0);
                let position = if gap_percent.abs() <= 5.0 {
                    "near_market"
                } else if gap_percent > 0.0 {
                    "premium"
                } else {
                    "below_market"
                };
                (Some(gap), Some(gap_percent), Some(position.to_string()))
            }
            _ => (None, None, None),
        };
    let lower_priced_competitors = product_price
        .map(|product_price| {
            competitor_prices
                .iter()
                .filter(|price| **price < product_price)
                .count()
        })
        .unwrap_or(0);
    let higher_priced_competitors = product_price
        .map(|product_price| {
            competitor_prices
                .iter()
                .filter(|price| **price > product_price)
                .count()
        })
        .unwrap_or(0);

    ProductCompetitorPriceAnalysisResponse {
        product_price,
        min_competitor_price,
        max_competitor_price,
        avg_competitor_price,
        price_position,
        gap_to_avg,
        gap_percent_to_avg,
        lower_priced_competitors,
        higher_priced_competitors,
    }
}

fn build_rating_analysis(ratings: &[f64]) -> ProductCompetitorRatingAnalysisResponse {
    ProductCompetitorRatingAnalysisResponse {
        min_rating: ratings.iter().copied().reduce(f64::min).map(round2),
        max_rating: ratings.iter().copied().reduce(f64::max).map(round2),
        avg_rating: average(ratings).map(round2),
        rated_competitors: ratings.len(),
    }
}

fn build_platform_analysis(
    competitors: &[ProductCompetitor],
) -> Vec<ProductCompetitorPlatformAnalysisResponse> {
    let mut platforms: BTreeMap<String, PlatformAccumulator> = BTreeMap::new();
    for competitor in competitors {
        let platform = competitor
            .platform
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        let entry = platforms.entry(platform).or_default();
        entry.competitors += 1;
        if let Some(price) = competitor
            .price
            .filter(|value| value.is_finite() && *value >= 0.0)
        {
            entry.price_sum += price;
            entry.price_count += 1;
        }
        if let Some(rating) = competitor
            .rating
            .filter(|value| value.is_finite() && *value >= 0.0)
        {
            entry.rating_sum += rating;
            entry.rating_count += 1;
        }
    }

    platforms
        .into_iter()
        .map(
            |(platform, entry)| ProductCompetitorPlatformAnalysisResponse {
                platform,
                competitors: entry.competitors,
                avg_price: (entry.price_count > 0)
                    .then(|| round2(entry.price_sum / entry.price_count as f64)),
                avg_rating: (entry.rating_count > 0)
                    .then(|| round2(entry.rating_sum / entry.rating_count as f64)),
            },
        )
        .collect()
}

fn analysis_readiness(counts: &ProductCompetitorAnalysisCounts) -> String {
    if counts.competitors == 0 {
        "empty".to_string()
    } else if counts.priced_competitors == 0 && counts.rated_competitors == 0 {
        "partial".to_string()
    } else {
        "ready".to_string()
    }
}

fn build_analysis_warnings(counts: &ProductCompetitorAnalysisCounts) -> Vec<String> {
    let mut warnings = Vec::new();
    if counts.competitors == 0 {
        warnings.push("no active competitors were found".to_string());
    }
    if counts.competitors > 0 && counts.priced_competitors == 0 {
        warnings.push("no competitor price data is available".to_string());
    }
    if counts.competitors > 0 && counts.recent_changes == 0 {
        warnings.push("no recent competitor changes were found".to_string());
    }
    warnings
}

fn build_analysis_signals(
    counts: &ProductCompetitorAnalysisCounts,
    price: &ProductCompetitorPriceAnalysisResponse,
    changes: &[CompetitorChange],
) -> Vec<String> {
    let mut signals = Vec::new();
    if counts.competitors == 0 {
        signals.push("competitor_pool_empty".to_string());
    }
    if let Some(position) = &price.price_position {
        signals.push(format!("price_position:{position}"));
    }
    if price.lower_priced_competitors > 0 {
        signals.push(format!(
            "lower_priced_competitors:{}",
            price.lower_priced_competitors
        ));
    }
    if changes
        .iter()
        .any(|change| change.change_type.contains("price"))
    {
        signals.push("recent_price_change".to_string());
    }
    if counts.monthly_sales_competitors == 0 && counts.competitors > 0 {
        signals.push("monthly_sales_missing".to_string());
    }
    signals
}

fn build_analysis_summary(
    readiness: &str,
    counts: &ProductCompetitorAnalysisCounts,
    price: &ProductCompetitorPriceAnalysisResponse,
) -> String {
    let price_part = match (&price.price_position, price.gap_percent_to_avg) {
        (Some(position), Some(gap)) => {
            format!(", product price is {gap:+.1}% vs competitor avg ({position})")
        }
        _ => String::new(),
    };
    format!(
        "Competitor analysis is {readiness}: {} competitor(s), {} priced, {} rated, {} recent change(s){}.",
        counts.competitors,
        counts.priced_competitors,
        counts.rated_competitors,
        counts.recent_changes,
        price_part
    )
}

fn build_analysis_refs(
    competitors: &[ProductCompetitorResponse],
    changes: &[CompetitorChangeResponse],
) -> Vec<ProductCompetitorAnalysisRefResponse> {
    competitors
        .iter()
        .map(|competitor| ProductCompetitorAnalysisRefResponse {
            ref_type: "product_competitor".to_string(),
            id: competitor.id.clone(),
            title: competitor.name.clone(),
            ref_key: format!("product_competitor:{}", competitor.id),
        })
        .chain(changes.iter().map(|change| {
            ProductCompetitorAnalysisRefResponse {
                ref_type: "competitor_change".to_string(),
                id: change.id.clone(),
                title: change
                    .summary
                    .clone()
                    .unwrap_or_else(|| change.change_type.clone()),
                ref_key: format!("competitor_change:{}", change.id),
            }
        }))
        .collect()
}

fn average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo::table::products::ProductStatus;
    use time::macros::datetime;

    #[test]
    fn competitor_analysis_marks_premium_price_position() {
        let product = product(Some("120".to_string()));
        let competitors = vec![
            competitor("A", Some(80.0), Some(4.5), Some("douyin")),
            competitor("B", Some(100.0), Some(4.7), Some("tmall")),
        ];
        let analysis = ProductCompetitorApi::build_competitor_analysis(
            product,
            competitors,
            Vec::new(),
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(analysis.readiness, "ready");
        assert_eq!(analysis.counts.competitors, 2);
        assert_eq!(analysis.price.avg_competitor_price, Some(90.0));
        assert_eq!(analysis.price.price_position.as_deref(), Some("premium"));
        assert_eq!(analysis.price.gap_percent_to_avg, Some(33.33));
    }

    #[test]
    fn competitor_analysis_empty_competitors_returns_empty_readiness() {
        let analysis = ProductCompetitorApi::build_competitor_analysis(
            product(None),
            Vec::new(),
            Vec::new(),
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(analysis.readiness, "empty");
        assert!(
            analysis
                .warnings
                .iter()
                .any(|item| item.contains("no active competitors"))
        );
    }

    fn product(price: Option<String>) -> Product {
        Product {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            platform: Some("douyin".to_string()),
            external_id: None,
            category: Some("coffee".to_string()),
            brand: Some("brand".to_string()),
            status: ProductStatus::Active,
            description: None,
            price,
            currency: Some("CNY".to_string()),
            metadata: json!({}),
            created_at: datetime!(2026-05-10 00:00 UTC),
            updated_at: datetime!(2026-05-10 00:00 UTC),
        }
    }

    fn competitor(
        name: &str,
        price: Option<f64>,
        rating: Option<f64>,
        platform: Option<&str>,
    ) -> ProductCompetitor {
        ProductCompetitor {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            name: name.to_string(),
            platform: platform.map(str::to_string),
            url: None,
            price,
            rating,
            monthly_sales: Some("1000+".to_string()),
            notes: None,
            search_results: json!({}),
            status: ProductCompetitorStatus::Active,
            metadata: json!({}),
            last_searched_at: None,
            created_at: datetime!(2026-05-10 00:00 UTC),
            updated_at: datetime!(2026-05-10 00:00 UTC),
        }
    }
}
