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
use serde::Deserialize;
use serde_json::{Map, Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use utils::llm_json::parse_llm_json;
use uuid::Uuid;

use crate::dto::product::{
    CreateProductDraftRequest, CreateProductRequest, DeleteProductRequest,
    GetProductLifecycleRequest, GetProductRequest, ImportProductTemplateRequest,
    ListProductsRequest, ProductDashboardResponse, ProductDetailResponse, ProductDraftResponse,
    ProductDraftValues, ProductLifecycleOptionsResponse, ProductLifecycleResponse,
    ProductLifecycleStatusResponse, ProductLifecycleTransitionResponse, ProductResponse,
    ProductTemplateImportResponse, ProductTemplateResponse, TransitionProductLifecycleRequest,
    UpdateProductRequest,
};

const DEFAULT_LIST_LIMIT: u64 = 100;
const PRODUCT_LIFECYCLE_HISTORY_LIMIT: usize = 20;
const PRODUCT_TEMPLATE_LIMIT: u64 = 500;

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

    pub async fn list_templates(&self, user_id: String) -> BizResult<Vec<ProductTemplateResponse>> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;

        Ok(product_templates()
            .into_iter()
            .map(product_template_response)
            .collect())
    }

    pub async fn import_template(
        &self,
        req: ImportProductTemplateRequest,
    ) -> BizResult<ProductTemplateImportResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let template = find_product_template(&req.template_id)?;

        if let Some(existing) = self
            .find_imported_template_product(user_id, template.id)
            .await?
        {
            return Ok(ProductTemplateImportResponse {
                product: Self::map_product(existing)?,
                created: false,
                template_id: template.id.to_string(),
            });
        }

        let product = self
            .product_svc
            .create(CreateProduct {
                id: Uuid::new_v4(),
                user_id,
                name: template.name.to_string(),
                platform: Some(template.platform.to_string()),
                external_id: Some(format!("demo-{}", template.id)),
                category: Some(template.category.to_string()),
                brand: Some(template.brand.to_string()),
                status: ProductStatus::Active,
                description: Some(template.description.to_string()),
                price: Some(template.price.to_string()),
                currency: Some(template.currency.to_string()),
                metadata: json!({
                    "source": "demo_template",
                    "templateId": template.id,
                    "importedAt": now_rfc3339(),
                }),
            })
            .await?;

        Ok(ProductTemplateImportResponse {
            product: Self::map_product(product)?,
            created: true,
            template_id: template.id.to_string(),
        })
    }

    pub async fn create_draft(
        &self,
        req: CreateProductDraftRequest,
    ) -> BizResult<ProductDraftResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;

        Ok(build_product_form_draft(req))
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

    async fn find_imported_template_product(
        &self,
        user_id: Uuid,
        template_id: &str,
    ) -> BizResult<Option<Product>> {
        let products = self
            .product_svc
            .list(ProductFilter {
                user_id,
                status: None,
                keyword: None,
                limit: Some(PRODUCT_TEMPLATE_LIMIT),
            })
            .await?;

        Ok(products
            .into_iter()
            .find(|product| product.metadata["templateId"].as_str() == Some(template_id)))
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

#[derive(Debug, Clone, Copy)]
struct ProductTemplate {
    id: &'static str,
    name: &'static str,
    platform: &'static str,
    category: &'static str,
    brand: &'static str,
    description: &'static str,
    price: &'static str,
    currency: &'static str,
}

fn product_templates() -> Vec<ProductTemplate> {
    vec![
        ProductTemplate {
            id: "thermos-student",
            name: "星河杯温显保温杯",
            platform: "douyin",
            category: "智能保温杯",
            brand: "星河杯",
            description: "面向学生和办公室人群的温显保温杯，主打防漏、温度提醒、通勤携带和礼品场景。",
            price: "69",
            currency: "CNY",
        },
        ProductTemplate {
            id: "coffee-drip-bag",
            name: "晨光庄园挂耳咖啡",
            platform: "taobao",
            category: "挂耳咖啡",
            brand: "晨光庄园",
            description: "适合宿舍、办公室和差旅场景的便携挂耳咖啡，突出产地风味、低门槛冲泡和复购组合。",
            price: "49",
            currency: "CNY",
        },
        ProductTemplate {
            id: "desk-lamp-eye-care",
            name: "青鹿护眼学习台灯",
            platform: "jd",
            category: "护眼台灯",
            brand: "青鹿",
            description: "面向学生学习桌面的护眼台灯，卖点包含无频闪、亮度调节、定时休息提醒和小桌面占用。",
            price: "129",
            currency: "CNY",
        },
    ]
}

fn find_product_template(template_id: &str) -> BizResult<ProductTemplate> {
    let template_id = normalize_required(template_id.to_string(), "template_id")?;
    product_templates()
        .into_iter()
        .find(|template| template.id == template_id)
        .ok_or_else(|| {
            BizError::new(
                app_error::PRODUCT_REQUEST_INVALID,
                format!("unknown product template: {template_id}"),
            )
        })
}

fn product_template_response(template: ProductTemplate) -> ProductTemplateResponse {
    ProductTemplateResponse {
        id: template.id.to_string(),
        name: template.name.to_string(),
        platform: Some(template.platform.to_string()),
        category: Some(template.category.to_string()),
        brand: Some(template.brand.to_string()),
        description: Some(template.description.to_string()),
        price: Some(template.price.to_string()),
        currency: Some(template.currency.to_string()),
    }
}

fn build_product_form_draft(req: CreateProductDraftRequest) -> ProductDraftResponse {
    let name = normalize_optional(req.name).unwrap_or_else(|| "智能保温杯训练商品".to_string());
    let category =
        normalize_optional(req.category).unwrap_or_else(|| infer_product_category(&name));
    let brand = normalize_optional(req.brand).unwrap_or_else(|| infer_product_brand(&name));
    let platform = normalize_optional(req.platform).unwrap_or_else(|| "douyin".to_string());
    let price = normalize_optional(req.price).unwrap_or_else(|| infer_product_price(&category));
    let currency = normalize_optional(req.currency).unwrap_or_else(|| "CNY".to_string());
    let prompt = normalize_optional(req.prompt);
    let description = normalize_optional(req.description)
        .unwrap_or_else(|| product_draft_description(&name, &category, prompt.as_deref()));
    let mut warnings = Vec::new();
    if prompt.is_none() {
        warnings.push("missing_prompt_used_template_draft".to_string());
    }

    ProductDraftResponse {
        draft: ProductDraftValues {
            name,
            platform: Some(platform),
            external_id: None,
            category: Some(category),
            brand: Some(brand),
            status: "active".to_string(),
            description: Some(description),
            price: Some(price),
            currency: Some(currency),
        },
        warnings,
        generation_mode: "template_smart_fill".to_string(),
        agent_context: json!({
            "type": "product_form_draft",
            "generationSource": "deterministic_template",
            "directWrite": false,
            "requiresUserReview": true,
            "llm": {
                "used": false,
                "futureRoute": "agent_session_sse"
            },
            "draftQuality": {
                "schema": "product_form_draft_v1",
                "fieldNormalization": "trim_empty_to_default",
                "invalidModelJsonPolicy": "reject_before_form_fill",
                "manualReviewRequired": true
            },
            "saveBoundary": {
                "directWrite": false,
                "requiresUserReview": true,
                "persistenceEndpoint": "POST /api/app/products"
            }
        }),
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(not(test), allow(dead_code))]
#[serde(rename_all = "camelCase")]
struct ProductFormDraftLlmCandidate {
    name: Option<String>,
    platform: Option<String>,
    external_id: Option<String>,
    category: Option<String>,
    brand: Option<String>,
    status: Option<String>,
    description: Option<String>,
    price: Option<String>,
    currency: Option<String>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_product_form_draft_from_llm(raw: &str) -> BizResult<ProductDraftValues> {
    let candidate = parse_llm_json::<ProductFormDraftLlmCandidate>(raw).map_err(|err| {
        BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            format!("invalid product form draft JSON: {err}"),
        )
    })?;

    if let Some(status) = normalize_optional(candidate.status) {
        let status = parse_status(status)?;
        if status != ProductStatus::Active {
            return Err(BizError::new(
                app_error::PRODUCT_REQUEST_INVALID,
                "product form draft status must be active".to_string(),
            ));
        }
    }

    let name = normalize_optional(candidate.name).ok_or_else(|| {
        BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            "product form draft name cannot be empty".to_string(),
        )
    })?;

    Ok(ProductDraftValues {
        name,
        platform: normalize_optional(candidate.platform),
        external_id: normalize_optional(candidate.external_id),
        category: normalize_optional(candidate.category),
        brand: normalize_optional(candidate.brand),
        status: ProductStatus::Active.as_str().to_string(),
        description: normalize_optional(candidate.description),
        price: normalize_optional(candidate.price),
        currency: normalize_optional(candidate.currency),
    })
}

fn infer_product_category(name: &str) -> String {
    if name.contains("咖啡") {
        "咖啡饮品".to_string()
    } else if name.contains("台灯") {
        "学习用品".to_string()
    } else if name.contains("杯") {
        "智能水杯".to_string()
    } else {
        "训练商品".to_string()
    }
}

fn infer_product_brand(name: &str) -> String {
    name.chars().take(4).collect::<String>().trim().to_string()
}

fn infer_product_price(category: &str) -> String {
    if category.contains("咖啡") {
        "49".to_string()
    } else if category.contains("学习") || category.contains("台灯") {
        "129".to_string()
    } else {
        "69".to_string()
    }
}

fn product_draft_description(name: &str, category: &str, prompt: Option<&str>) -> String {
    let prompt_hint = prompt
        .map(|value| format!(" 已结合补充信息：{}。", truncate_chars(value, 120)))
        .unwrap_or_default();
    format!(
        "{name} 是用于电商实训的{category}商品草稿，建议围绕目标人群、核心卖点、价格带、使用场景和竞品差异继续补充资料。{prompt_hint}"
    )
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
        history.drain(0..overflow);
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

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
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

    #[test]
    fn product_form_draft_keeps_user_review_boundary() {
        let draft = build_product_form_draft(CreateProductDraftRequest {
            user_id: "11111111-1111-1111-1111-111111111111".to_string(),
            name: Some("温显杯".to_string()),
            platform: None,
            category: None,
            brand: None,
            description: None,
            price: None,
            currency: None,
            prompt: Some("面向学生，突出防漏和温度显示".to_string()),
        });

        assert_eq!(draft.draft.name, "温显杯");
        assert_eq!(draft.draft.status, "active");
        assert_eq!(draft.generation_mode, "template_smart_fill");
        assert_eq!(
            draft.agent_context["requiresUserReview"].as_bool(),
            Some(true)
        );
        assert_eq!(
            draft.agent_context["generationSource"].as_str(),
            Some("deterministic_template")
        );
        assert_eq!(draft.agent_context["llm"]["used"].as_bool(), Some(false));
        assert_eq!(
            draft.agent_context["draftQuality"]["schema"].as_str(),
            Some("product_form_draft_v1")
        );
        assert_eq!(
            draft.agent_context["draftQuality"]["fieldNormalization"].as_str(),
            Some("trim_empty_to_default")
        );
        assert_eq!(
            draft.agent_context["draftQuality"]["invalidModelJsonPolicy"].as_str(),
            Some("reject_before_form_fill")
        );
        assert_eq!(
            draft.agent_context["saveBoundary"]["persistenceEndpoint"].as_str(),
            Some("POST /api/app/products")
        );
        assert!(draft.draft.description.unwrap().contains("温显杯"));
    }

    #[test]
    fn product_form_draft_llm_json_accepts_fenced_json_and_server_status() {
        let draft = parse_product_form_draft_from_llm(
            "```json\n{\"name\":\" 温显杯 \",\"platform\":\"douyin\",\"status\":\"active\",\"description\":\" 防漏温显 \"}\n```",
        )
        .expect("draft parsed");

        assert_eq!(draft.name, "温显杯");
        assert_eq!(draft.platform.as_deref(), Some("douyin"));
        assert_eq!(draft.status, "active");
        assert_eq!(draft.description.as_deref(), Some("防漏温显"));
    }

    #[test]
    fn product_form_draft_llm_json_rejects_invalid_status_before_fill() {
        let err = parse_product_form_draft_from_llm(
            r#"{"name":"温显杯","status":"archived","description":"test"}"#,
        )
        .unwrap_err();

        assert_eq!(err.code(), app_error::PRODUCT_REQUEST_INVALID);
    }

    #[test]
    fn product_form_draft_llm_json_rejects_bad_json_before_fill() {
        let err = parse_product_form_draft_from_llm("模型输出：name=温显杯").unwrap_err();

        assert_eq!(err.code(), app_error::PRODUCT_REQUEST_INVALID);
    }

    #[test]
    fn product_form_draft_llm_json_rejects_empty_required_name() {
        let err =
            parse_product_form_draft_from_llm(r#"{"name":"   ","status":"active"}"#).unwrap_err();

        assert_eq!(err.code(), app_error::PRODUCT_REQUEST_INVALID);
    }

    #[test]
    fn product_templates_are_available_for_demo_import() {
        let templates = product_templates();

        assert!(templates.len() >= 3);
        assert!(
            templates
                .iter()
                .any(|template| template.id == "thermos-student")
        );
    }
}
