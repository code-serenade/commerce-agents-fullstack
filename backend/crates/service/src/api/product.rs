use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    products::{
        CreateProduct, Product, ProductFilter, ProductService, ProductStatus, UpdateProduct,
    },
};
use serde_json::{Map, Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::dto::product::{
    CreateProductRequest, DeleteProductRequest, GetProductLifecycleRequest, GetProductRequest,
    ListProductsRequest, ProductDashboardResponse, ProductDetailResponse,
    ProductLifecycleOptionsResponse, ProductLifecycleResponse, ProductLifecycleStatusResponse,
    ProductLifecycleTransitionResponse, ProductResponse, TransitionProductLifecycleRequest,
    UpdateProductRequest,
};

const DEFAULT_LIST_LIMIT: u64 = 100;
const PRODUCT_LIFECYCLE_HISTORY_LIMIT: usize = 20;

pub struct ProductApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
}

impl ProductApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db),
        }
    }

    pub async fn list_products(&self, req: ListProductsRequest) -> BizResult<Vec<ProductResponse>> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let status = parse_optional_status(req.status)?;

        self.product_svc
            .list(ProductFilter {
                user_id,
                status: Some(status.unwrap_or(ProductStatus::Active)),
                keyword: normalize_optional(req.keyword),
                limit: Some(DEFAULT_LIST_LIMIT),
            })
            .await?
            .into_iter()
            .map(Self::map_product)
            .collect()
    }

    pub async fn create_product(&self, req: CreateProductRequest) -> BizResult<ProductResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let name = normalize_required(req.name, "name")?;
        let status = parse_optional_status(req.status)?.unwrap_or(ProductStatus::Active);

        let product = self
            .product_svc
            .create(CreateProduct {
                id: Uuid::new_v4(),
                user_id,
                name,
                platform: normalize_optional(req.platform),
                external_id: normalize_optional(req.external_id),
                category: normalize_optional(req.category),
                brand: normalize_optional(req.brand),
                status,
                description: normalize_optional(req.description),
                price: normalize_optional(req.price),
                currency: normalize_optional(req.currency),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_product(product)
    }

    pub async fn get_product(&self, req: GetProductRequest) -> BizResult<ProductDetailResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let product = self.ensure_product_owner(user_id, product_id).await?;

        Ok(ProductDetailResponse {
            product: Self::map_product(product)?,
        })
    }

    pub async fn update_product(&self, req: UpdateProductRequest) -> BizResult<ProductResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let current = self
            .ensure_active_product_owner(user_id, product_id)
            .await?;

        let product = self
            .product_svc
            .update(UpdateProduct {
                id: current.id,
                name: normalize_optional(req.name).unwrap_or(current.name),
                platform: req
                    .platform
                    .map(normalize_string)
                    .unwrap_or(current.platform),
                external_id: req
                    .external_id
                    .map(normalize_string)
                    .unwrap_or(current.external_id),
                category: req
                    .category
                    .map(normalize_string)
                    .unwrap_or(current.category),
                brand: req.brand.map(normalize_string).unwrap_or(current.brand),
                status: current.status,
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                price: req.price.map(normalize_string).unwrap_or(current.price),
                currency: req
                    .currency
                    .map(normalize_string)
                    .unwrap_or(current.currency),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;

        Self::map_product(product)
    }

    pub async fn delete_product(&self, req: DeleteProductRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let current = self
            .ensure_active_product_owner(user_id, product_id)
            .await?;

        self.product_svc
            .update(UpdateProduct {
                id: current.id,
                name: current.name,
                platform: current.platform,
                external_id: current.external_id,
                category: current.category,
                brand: current.brand,
                status: ProductStatus::Archived,
                description: current.description,
                price: current.price,
                currency: current.currency,
                metadata: current.metadata,
            })
            .await?;

        Ok(())
    }

    pub async fn lifecycle_options(
        &self,
        user_id: String,
    ) -> BizResult<ProductLifecycleOptionsResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        Ok(build_lifecycle_options_response())
    }

    pub async fn get_lifecycle(
        &self,
        req: GetProductLifecycleRequest,
    ) -> BizResult<ProductLifecycleResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let product = self.ensure_product_owner(user_id, product_id).await?;

        self.build_lifecycle_response(product)
    }

    pub async fn transition_lifecycle(
        &self,
        req: TransitionProductLifecycleRequest,
    ) -> BizResult<ProductLifecycleResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_uuid(&req.product_id, "product_id")?;
        let current = self.ensure_product_owner(user_id, product_id).await?;
        let target_status = parse_required_status(req.target_status)?;

        validate_lifecycle_transition(current.status, target_status)?;

        let product = self
            .product_svc
            .update(UpdateProduct {
                id: current.id,
                name: current.name,
                platform: current.platform,
                external_id: current.external_id,
                category: current.category,
                brand: current.brand,
                status: target_status,
                description: current.description,
                price: current.price,
                currency: current.currency,
                metadata: merge_lifecycle_metadata(
                    current.metadata,
                    current.status,
                    target_status,
                    normalize_optional(req.reason),
                    req.metadata.unwrap_or_else(|| json!({})),
                ),
            })
            .await?;

        self.build_lifecycle_response(product)
    }

    pub async fn dashboard(&self, user_id: String) -> BizResult<ProductDashboardResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let draft = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Draft)
            .await?;
        let listing = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Listing)
            .await?;
        let active = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Active)
            .await?;
        let archived = self
            .product_svc
            .count_by_user_and_status(user_id, ProductStatus::Archived)
            .await?;

        Ok(ProductDashboardResponse {
            total: draft + listing + active + archived,
            draft,
            listing,
            active,
            archived,
        })
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

    fn map_product(item: Product) -> BizResult<ProductResponse> {
        Ok(ProductResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            name: item.name,
            platform: item.platform,
            external_id: item.external_id,
            category: item.category,
            brand: item.brand,
            status: item.status.as_str().to_string(),
            description: item.description,
            price: item.price,
            currency: item.currency,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }

    fn build_lifecycle_response(&self, product: Product) -> BizResult<ProductLifecycleResponse> {
        let current_status = product.status;
        let product_id = product.id.to_string();
        let allowed_next_statuses = allowed_next_statuses(current_status)
            .into_iter()
            .map(|status| status.as_str().to_string())
            .collect::<Vec<_>>();
        let transitions = lifecycle_transitions()
            .into_iter()
            .filter(|transition| transition.from == current_status)
            .map(map_lifecycle_transition)
            .collect::<Vec<_>>();
        let warnings = lifecycle_warnings(current_status);

        Ok(ProductLifecycleResponse {
            product: Self::map_product(product)?,
            current_status: current_status.as_str().to_string(),
            allowed_next_statuses: allowed_next_statuses.clone(),
            transitions,
            warnings,
            agent_context: json!({
                "type": "product_lifecycle",
                "productId": product_id,
                "currentStatus": current_status.as_str(),
                "allowedNextStatuses": allowed_next_statuses,
                "terminal": current_status == ProductStatus::Archived,
            }),
        })
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_status(value: Option<String>) -> BizResult<Option<ProductStatus>> {
    value
        .and_then(normalize_string)
        .map(parse_status)
        .transpose()
}

fn parse_required_status(value: String) -> BizResult<ProductStatus> {
    let value = normalize_required(value, "target_status")?;
    parse_status(value)
}

fn parse_status(value: String) -> BizResult<ProductStatus> {
    match value.as_str() {
        "draft" => Ok(ProductStatus::Draft),
        "listing" => Ok(ProductStatus::Listing),
        "active" => Ok(ProductStatus::Active),
        "archived" => Ok(ProductStatus::Archived),
        _ => Err(BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            format!("invalid product status: {value}"),
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LifecycleTransition {
    from: ProductStatus,
    to: ProductStatus,
    label: &'static str,
    requires_reason: bool,
}

fn lifecycle_statuses() -> Vec<ProductLifecycleStatusResponse> {
    vec![
        ProductLifecycleStatusResponse {
            status: "draft".to_string(),
            label: "选品中".to_string(),
            description: "商品仍在选品、定位或资料准备阶段。".to_string(),
            terminal: false,
        },
        ProductLifecycleStatusResponse {
            status: "listing".to_string(),
            label: "上架中".to_string(),
            description: "商品正在准备上架资料、素材、价格和渠道配置。".to_string(),
            terminal: false,
        },
        ProductLifecycleStatusResponse {
            status: "active".to_string(),
            label: "运营中".to_string(),
            description: "商品已进入日常运营、投放、内容和复盘阶段。".to_string(),
            terminal: false,
        },
        ProductLifecycleStatusResponse {
            status: "archived".to_string(),
            label: "已归档".to_string(),
            description: "商品已归档，不再参与新的运营动作。".to_string(),
            terminal: true,
        },
    ]
}

fn lifecycle_transitions() -> Vec<LifecycleTransition> {
    vec![
        LifecycleTransition {
            from: ProductStatus::Draft,
            to: ProductStatus::Listing,
            label: "进入上架准备",
            requires_reason: false,
        },
        LifecycleTransition {
            from: ProductStatus::Listing,
            to: ProductStatus::Active,
            label: "进入运营",
            requires_reason: false,
        },
        LifecycleTransition {
            from: ProductStatus::Listing,
            to: ProductStatus::Draft,
            label: "退回选品",
            requires_reason: true,
        },
        LifecycleTransition {
            from: ProductStatus::Active,
            to: ProductStatus::Archived,
            label: "归档商品",
            requires_reason: true,
        },
    ]
}

fn map_lifecycle_transition(transition: LifecycleTransition) -> ProductLifecycleTransitionResponse {
    ProductLifecycleTransitionResponse {
        from: transition.from.as_str().to_string(),
        to: transition.to.as_str().to_string(),
        label: transition.label.to_string(),
        requires_reason: transition.requires_reason,
    }
}

fn build_lifecycle_options_response() -> ProductLifecycleOptionsResponse {
    ProductLifecycleOptionsResponse {
        statuses: lifecycle_statuses(),
        transitions: lifecycle_transitions()
            .into_iter()
            .map(map_lifecycle_transition)
            .collect(),
        agent_context: json!({
            "type": "product_lifecycle_options",
            "statuses": ["draft", "listing", "active", "archived"],
            "terminalStatuses": ["archived"],
        }),
    }
}

fn allowed_next_statuses(status: ProductStatus) -> Vec<ProductStatus> {
    lifecycle_transitions()
        .into_iter()
        .filter(|transition| transition.from == status)
        .map(|transition| transition.to)
        .collect()
}

fn validate_lifecycle_transition(from: ProductStatus, to: ProductStatus) -> BizResult<()> {
    if allowed_next_statuses(from).contains(&to) {
        return Ok(());
    }

    Err(BizError::new(
        app_error::PRODUCT_REQUEST_INVALID,
        format!(
            "invalid product lifecycle transition: {} -> {}",
            from.as_str(),
            to.as_str()
        ),
    ))
}

fn lifecycle_warnings(status: ProductStatus) -> Vec<String> {
    if status == ProductStatus::Archived {
        return vec!["product is archived; lifecycle is terminal".to_string()];
    }
    Vec::new()
}

fn merge_lifecycle_metadata(
    current: Value,
    from: ProductStatus,
    to: ProductStatus,
    reason: Option<String>,
    transition_metadata: Value,
) -> Value {
    let mut object = current.as_object().cloned().unwrap_or_default();
    let mut lifecycle = match object.remove("lifecycle") {
        Some(Value::Object(existing)) => existing,
        _ => Map::new(),
    };

    let transition = json!({
        "from": from.as_str(),
        "to": to.as_str(),
        "reason": reason,
        "metadata": transition_metadata,
        "source": "api",
        "updatedAt": now_rfc3339(),
    });

    let mut history = match lifecycle.remove("history") {
        Some(Value::Array(existing)) => existing,
        _ => Vec::new(),
    };
    history.push(transition.clone());
    if history.len() > PRODUCT_LIFECYCLE_HISTORY_LIMIT {
        let overflow = history.len() - PRODUCT_LIFECYCLE_HISTORY_LIMIT;
        history.drain(0 .. overflow);
    }

    lifecycle.insert(
        "currentStatus".to_string(),
        Value::String(to.as_str().to_string()),
    );
    lifecycle.insert("lastTransition".to_string(), transition);
    lifecycle.insert("history".to_string(), Value::Array(history));
    object.insert("lifecycle".to_string(), Value::Object(lifecycle));

    Value::Object(object)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_lifecycle_allows_expected_transitions() {
        assert!(
            validate_lifecycle_transition(ProductStatus::Draft, ProductStatus::Listing).is_ok()
        );
        assert!(
            validate_lifecycle_transition(ProductStatus::Listing, ProductStatus::Active).is_ok()
        );
        assert!(
            validate_lifecycle_transition(ProductStatus::Listing, ProductStatus::Draft).is_ok()
        );
        assert!(
            validate_lifecycle_transition(ProductStatus::Active, ProductStatus::Archived).is_ok()
        );
    }

    #[test]
    fn product_lifecycle_rejects_invalid_transition() {
        let err =
            validate_lifecycle_transition(ProductStatus::Draft, ProductStatus::Active).unwrap_err();

        assert_eq!(err.code(), app_error::PRODUCT_REQUEST_INVALID);
    }

    #[test]
    fn product_lifecycle_archived_is_terminal() {
        let err = validate_lifecycle_transition(ProductStatus::Archived, ProductStatus::Active)
            .unwrap_err();

        assert_eq!(err.code(), app_error::PRODUCT_REQUEST_INVALID);
        assert!(allowed_next_statuses(ProductStatus::Archived).is_empty());
    }

    #[test]
    fn product_lifecycle_metadata_records_last_transition() {
        let metadata = merge_lifecycle_metadata(
            json!({"source": "seed"}),
            ProductStatus::Draft,
            ProductStatus::Listing,
            Some("ready".to_string()),
            json!({"operator": "test"}),
        );

        assert_eq!(metadata["source"], "seed");
        assert_eq!(metadata["lifecycle"]["currentStatus"], "listing");
        assert_eq!(metadata["lifecycle"]["lastTransition"]["from"], "draft");
        assert_eq!(metadata["lifecycle"]["lastTransition"]["to"], "listing");
        assert_eq!(metadata["lifecycle"]["lastTransition"]["reason"], "ready");
        assert_eq!(
            metadata["lifecycle"]["lastTransition"]["metadata"]["operator"],
            "test"
        );
        assert_eq!(
            metadata["lifecycle"]["history"].as_array().unwrap().len(),
            1
        );
    }
}
