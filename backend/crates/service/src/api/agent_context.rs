use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    product_knowledge::{
        ProductKnowledge, ProductKnowledgeFilter, ProductKnowledgeSearchFilter,
        ProductKnowledgeService, ProductKnowledgeStatus,
    },
    products::{Product, ProductFilter, ProductService, ProductStatus},
};
use uuid::Uuid;

use crate::dto::agent_context::{
    AgentContextSource, AgentProductContextResponse, AgentRetrievalContextResponse,
    BuildAgentProductContextRequest, BuildAgentRetrievalContextRequest,
};

const MAX_CONTEXT_PRODUCTS: usize = 5;
const MAX_CONTEXT_REFS: usize = 20;
const KNOWLEDGE_PER_PRODUCT_LIMIT: u64 = 6;
const MAX_CONTEXT_SUMMARY_CHARS: usize = 6000;
const MAX_KNOWLEDGE_CONTENT_CHARS: usize = 700;
const DEFAULT_RETRIEVAL_LIMIT: usize = 8;
const MAX_RETRIEVAL_LIMIT: usize = 12;
const MAX_QUERY_TERMS: usize = 12;
const MAX_RETRIEVAL_PRODUCTS: usize = 6;
const MAX_RETRIEVAL_SUMMARY_CHARS: usize = 5000;
const LOCAL_RAG_MODE: &str = "local_product_knowledge";
const EXTERNAL_SEARCH_UNAVAILABLE_REASON: &str =
    "No matching local product knowledge was found; external search adapter is not configured.";
const EXTERNAL_SEARCH_AVAILABLE_REASON: &str = "No matching local product knowledge was found; external search adapter is configured and \
     search skills may request live sources when the task needs real-time market data.";

pub struct AgentContextApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    knowledge_svc: ProductKnowledgeService,
}

impl AgentContextApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            knowledge_svc: ProductKnowledgeService::new(db),
        }
    }

    pub async fn build_product_context(
        &self,
        req: BuildAgentProductContextRequest,
    ) -> BizResult<AgentProductContextResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let context_refs = normalize_context_refs(req.context_refs)?;
        let product_ids = normalize_product_ids(req.product_ids, &context_refs)?;

        let mut resolved_products = Vec::new();
        let mut resolved_knowledge = Vec::new();
        let mut sources = Vec::new();

        for product_id in product_ids {
            let product = self
                .ensure_active_product_owner(user_id, product_id)
                .await?;
            sources.push(AgentContextSource {
                source_type: "product".to_string(),
                id: product.id.to_string(),
                product_id: Some(product.id.to_string()),
                title: product.name.clone(),
            });

            let knowledge = self
                .knowledge_svc
                .list(ProductKnowledgeFilter {
                    user_id,
                    product_id: product.id,
                    status: Some(ProductKnowledgeStatus::Active),
                    content_type: None,
                    keyword: None,
                    limit: Some(KNOWLEDGE_PER_PRODUCT_LIMIT),
                })
                .await?;
            for item in &knowledge {
                sources.push(AgentContextSource {
                    source_type: "product_knowledge".to_string(),
                    id: item.id.to_string(),
                    product_id: Some(item.product_id.to_string()),
                    title: item
                        .title
                        .clone()
                        .unwrap_or_else(|| item.content_type.clone()),
                });
            }
            resolved_knowledge.extend(knowledge);
            resolved_products.push(product);
        }

        let summary = build_context_summary(&resolved_products, &resolved_knowledge, &context_refs);

        Ok(AgentProductContextResponse {
            product_ids: resolved_products
                .iter()
                .map(|product| product.id.to_string())
                .collect(),
            context_refs,
            product_count: resolved_products.len(),
            knowledge_count: resolved_knowledge.len(),
            summary,
            sources,
        })
    }

    pub async fn build_retrieval_context(
        &self,
        req: BuildAgentRetrievalContextRequest,
    ) -> BizResult<AgentRetrievalContextResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let context_refs = normalize_context_refs(req.context_refs)?;
        let explicit_product_ids = normalize_product_ids(req.product_ids, &context_refs)?;
        let query_terms = extract_query_terms(&req.query);
        let limit = req
            .limit
            .unwrap_or(DEFAULT_RETRIEVAL_LIMIT)
            .clamp(1, MAX_RETRIEVAL_LIMIT);

        let mut resolved_products = Vec::new();
        let mut resolved_knowledge = Vec::new();

        for product_id in explicit_product_ids {
            let product = self
                .ensure_active_product_owner(user_id, product_id)
                .await?;
            push_unique_product(&mut resolved_products, product);
        }

        if req.allow_implicit_products {
            for term in &query_terms {
                if resolved_products.len() >= MAX_RETRIEVAL_PRODUCTS {
                    break;
                }
                let products = self
                    .product_svc
                    .list(ProductFilter {
                        user_id,
                        status: Some(ProductStatus::Active),
                        keyword: Some(term.clone()),
                        limit: Some(MAX_RETRIEVAL_PRODUCTS as u64),
                    })
                    .await?;
                for product in products {
                    if resolved_products.len() >= MAX_RETRIEVAL_PRODUCTS {
                        break;
                    }
                    push_unique_product(&mut resolved_products, product);
                }
            }
        }

        for product in &resolved_products {
            if resolved_knowledge.len() >= limit {
                break;
            }
            let mut product_knowledge = self
                .knowledge_svc
                .list(ProductKnowledgeFilter {
                    user_id,
                    product_id: product.id,
                    status: Some(ProductKnowledgeStatus::Active),
                    content_type: None,
                    keyword: None,
                    limit: Some(KNOWLEDGE_PER_PRODUCT_LIMIT),
                })
                .await?;
            for item in product_knowledge.drain(..) {
                if resolved_knowledge.len() >= limit {
                    break;
                }
                push_unique_knowledge(&mut resolved_knowledge, item);
            }
        }

        if req.allow_implicit_products {
            for term in &query_terms {
                if resolved_knowledge.len() >= limit {
                    break;
                }
                let knowledge = self
                    .knowledge_svc
                    .search_by_user(ProductKnowledgeSearchFilter {
                        user_id,
                        status: Some(ProductKnowledgeStatus::Active),
                        keyword: Some(term.clone()),
                        limit: Some(limit as u64),
                    })
                    .await?;
                for item in knowledge {
                    if resolved_knowledge.len() >= limit {
                        break;
                    }
                    let Ok(product) = self
                        .ensure_active_product_owner(user_id, item.product_id)
                        .await
                    else {
                        continue;
                    };
                    push_unique_product(&mut resolved_products, product);
                    push_unique_knowledge(&mut resolved_knowledge, item);
                }
            }
        }

        let sources = build_retrieval_sources(&resolved_products, &resolved_knowledge);
        let summary =
            build_retrieval_summary(&resolved_products, &resolved_knowledge, &query_terms);
        let source_count = sources.len();
        let degraded = source_count == 0;
        let external_search_available = req.external_search_available;

        Ok(AgentRetrievalContextResponse {
            mode: LOCAL_RAG_MODE.to_string(),
            query_terms,
            product_ids: resolved_products
                .iter()
                .map(|product| product.id.to_string())
                .collect(),
            knowledge_count: resolved_knowledge.len(),
            source_count,
            summary,
            sources,
            degraded,
            degraded_reason: degraded.then(|| retrieval_degraded_reason(external_search_available)),
            external_search_available,
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

fn normalize_context_refs(context_refs: Vec<String>) -> BizResult<Vec<String>> {
    if context_refs.len() > MAX_CONTEXT_REFS {
        return Err(BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            format!("context_refs cannot exceed {MAX_CONTEXT_REFS} items"),
        ));
    }

    let mut refs = Vec::new();
    for item in context_refs {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if item.chars().count() > 160 {
            return Err(BizError::new(
                app_error::PRODUCT_REQUEST_INVALID,
                "context ref must be at most 160 characters".to_string(),
            ));
        }
        push_unique(&mut refs, item.to_string());
    }
    Ok(refs)
}

fn normalize_product_ids(
    product_ids: Vec<String>,
    context_refs: &[String],
) -> BizResult<Vec<Uuid>> {
    let mut values = Vec::new();
    for product_id in product_ids {
        let product_id = product_id.trim();
        if product_id.is_empty() {
            continue;
        }
        let parsed = parse_uuid(product_id, "product_id")?;
        push_unique_uuid(&mut values, parsed);
    }

    for context_ref in context_refs {
        if let Some(product_id) = product_id_from_context_ref(context_ref) {
            let parsed = parse_uuid(product_id, "context_ref product id")?;
            push_unique_uuid(&mut values, parsed);
        }
    }

    if values.len() > MAX_CONTEXT_PRODUCTS {
        return Err(BizError::new(
            app_error::PRODUCT_REQUEST_INVALID,
            format!("product_ids cannot exceed {MAX_CONTEXT_PRODUCTS} items"),
        ));
    }
    Ok(values)
}

fn product_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("product:")
        .or_else(|| value.strip_prefix("product/"))
}

fn build_context_summary(
    products: &[Product],
    knowledge: &[ProductKnowledge],
    context_refs: &[String],
) -> String {
    if products.is_empty() && context_refs.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("Commerce product context resolved from current user's backend data.".to_string());
    lines.push(
        "Use these facts when relevant, cite product or knowledge titles, and do not invent \
         missing product details."
            .to_string(),
    );

    for product in products {
        lines.push(format!(
            "Product source product:{} | name={} | platform={} | category={} | brand={} | \
             price={} {} | status={} | description={}",
            product.id,
            product.name,
            option_text(&product.platform),
            option_text(&product.category),
            option_text(&product.brand),
            option_text(&product.price),
            option_text(&product.currency),
            product.status.as_str(),
            option_text(&product.description)
        ));

        for item in knowledge
            .iter()
            .filter(|item| item.product_id == product.id)
        {
            lines.push(format!(
                "Knowledge source product_knowledge:{} | product={} | title={} | type={} | \
                 source={} | confidence={:.2} | content={}",
                item.id,
                product.id,
                option_text(&item.title),
                item.content_type,
                source_label(item),
                item.confidence,
                compact_text(&item.content, MAX_KNOWLEDGE_CONTENT_CHARS)
            ));
        }
    }

    if !context_refs.is_empty() {
        lines.push(format!(
            "Requested context refs: {}",
            context_refs.join(", ")
        ));
    }

    compact_text(&lines.join("\n"), MAX_CONTEXT_SUMMARY_CHARS)
}

fn build_retrieval_sources(
    products: &[Product],
    knowledge: &[ProductKnowledge],
) -> Vec<AgentContextSource> {
    let mut sources = Vec::new();
    for product in products {
        if knowledge.iter().any(|item| item.product_id == product.id) {
            sources.push(AgentContextSource {
                source_type: "product".to_string(),
                id: product.id.to_string(),
                product_id: Some(product.id.to_string()),
                title: product.name.clone(),
            });
        }
    }
    for item in knowledge {
        sources.push(AgentContextSource {
            source_type: "product_knowledge".to_string(),
            id: item.id.to_string(),
            product_id: Some(item.product_id.to_string()),
            title: item
                .title
                .clone()
                .unwrap_or_else(|| item.content_type.clone()),
        });
    }
    sources
}

fn build_retrieval_summary(
    products: &[Product],
    knowledge: &[ProductKnowledge],
    query_terms: &[String],
) -> String {
    if knowledge.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push(
        "Local commerce RAG context retrieved from current user's product knowledge.".to_string(),
    );
    lines.push(
        "Use these facts when relevant, cite product or knowledge titles, and do not invent \
         external market facts."
            .to_string(),
    );
    if !query_terms.is_empty() {
        lines.push(format!("Retrieval query terms: {}", query_terms.join(", ")));
    }

    for item in knowledge {
        let product = products
            .iter()
            .find(|product| product.id == item.product_id);
        lines.push(format!(
            "Retrieved knowledge source product_knowledge:{} | product={} | product_name={} | \
             title={} | type={} | source={} | confidence={:.2} | content={}",
            item.id,
            item.product_id,
            product.map(|item| item.name.as_str()).unwrap_or("unknown"),
            option_text(&item.title),
            item.content_type,
            source_label(item),
            item.confidence,
            compact_text(&item.content, MAX_KNOWLEDGE_CONTENT_CHARS)
        ));
    }

    compact_text(&lines.join("\n"), MAX_RETRIEVAL_SUMMARY_CHARS)
}

fn retrieval_degraded_reason(external_search_available: bool) -> String {
    if external_search_available {
        EXTERNAL_SEARCH_AVAILABLE_REASON.to_string()
    } else {
        EXTERNAL_SEARCH_UNAVAILABLE_REASON.to_string()
    }
}

fn extract_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let normalized = query.trim();
    for token in normalized
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .map(str::trim)
        .filter(|token| token.chars().count() >= 2)
    {
        push_unique_limited(&mut terms, token.to_ascii_lowercase(), MAX_QUERY_TERMS);
    }

    let lower = normalized.to_ascii_lowercase();
    for term in COMMERCE_QUERY_TERMS {
        if normalized.contains(term) || lower.contains(&term.to_ascii_lowercase()) {
            push_unique_limited(&mut terms, (*term).to_string(), MAX_QUERY_TERMS);
        }
    }

    if terms.is_empty() && contains_cjk(normalized) {
        for token in cjk_windows(normalized, 2) {
            push_unique_limited(&mut terms, token, MAX_QUERY_TERMS);
            if terms.len() >= MAX_QUERY_TERMS {
                break;
            }
        }
    }

    terms
}

const COMMERCE_QUERY_TERMS: &[&str] = &[
    "商品",
    "产品",
    "咖啡",
    "奶茶",
    "茶饮",
    "零食",
    "美妆",
    "服饰",
    "母婴",
    "家居",
    "价格",
    "定价",
    "库存",
    "活动",
    "促销",
    "投放",
    "广告",
    "千川",
    "抖音",
    "小红书",
    "淘宝",
    "天猫",
    "京东",
    "拼多多",
    "转化",
    "复购",
    "竞品",
    "客诉",
    "退款",
    "售后",
    "物流",
    "主图",
    "详情页",
    "标题",
    "关键词",
    "直播",
    "短视频",
    "素材",
    "毛利",
    "利润",
    "roi",
    "预算",
    "复盘",
    "人群",
    "卖点",
    "五一",
    "618",
    "双11",
];

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

fn cjk_windows(value: &str, window: usize) -> Vec<String> {
    let chars = value
        .chars()
        .filter(|ch| ('\u{4e00}'..='\u{9fff}').contains(ch))
        .collect::<Vec<_>>();
    if chars.len() < window {
        return Vec::new();
    }
    chars
        .windows(window)
        .map(|items| items.iter().collect::<String>())
        .collect()
}

fn option_text(value: &Option<String>) -> &str {
    value
        .as_deref()
        .filter(|item| !item.trim().is_empty())
        .unwrap_or("unknown")
}

fn source_label(item: &ProductKnowledge) -> String {
    match item
        .source_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        Some(source_id) => format!("{}:{source_id}", item.source_type),
        None => item.source_type.clone(),
    }
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if !items.iter().any(|item| item == &value) {
        items.push(value);
    }
}

fn push_unique_limited(items: &mut Vec<String>, value: String, limit: usize) {
    if items.len() >= limit {
        return;
    }
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn push_unique_product(items: &mut Vec<Product>, product: Product) {
    if !items.iter().any(|item| item.id == product.id) {
        items.push(product);
    }
}

fn push_unique_knowledge(items: &mut Vec<ProductKnowledge>, knowledge: ProductKnowledge) {
    if !items.iter().any(|item| item.id == knowledge.id) {
        items.push(knowledge);
    }
}

#[cfg(test)]
mod tests {
    use repo::table::{
        product_knowledge::{ProductKnowledge, ProductKnowledgeStatus},
        products::{Product, ProductStatus},
    };
    use serde_json::json;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{
        build_context_summary, build_retrieval_sources, build_retrieval_summary,
        extract_query_terms, normalize_context_refs, normalize_product_ids,
        retrieval_degraded_reason,
    };

    #[test]
    fn context_refs_can_select_product_ids() {
        let product_id = Uuid::new_v4();
        let ids = normalize_product_ids(
            vec![product_id.to_string()],
            &[format!("product:{product_id}")],
        )
        .expect("product ids should normalize");

        assert_eq!(ids, vec![product_id]);
    }

    #[test]
    fn rejects_too_many_context_refs() {
        let refs = (0..21).map(|index| format!("ref-{index}")).collect();

        let err = normalize_context_refs(refs).expect_err("too many refs should fail");

        assert_eq!(err.code(), error_code::app::PRODUCT_REQUEST_INVALID);
    }

    #[test]
    fn summary_contains_product_and_knowledge_sources() {
        let product_id = Uuid::new_v4();
        let knowledge_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let summary = build_context_summary(
            &[Product {
                id: product_id,
                user_id: Uuid::new_v4(),
                name: "五一咖啡组合".to_string(),
                platform: Some("douyin".to_string()),
                external_id: None,
                category: Some("coffee".to_string()),
                brand: Some("Atlas".to_string()),
                status: ProductStatus::Active,
                description: Some("节日主推组合".to_string()),
                price: Some("99".to_string()),
                currency: Some("CNY".to_string()),
                metadata: json!({}),
                created_at: now,
                updated_at: now,
            }],
            &[ProductKnowledge {
                id: knowledge_id,
                user_id: Uuid::new_v4(),
                product_id,
                title: Some("核心卖点".to_string()),
                content_type: "selling_point".to_string(),
                content: "低糖、便携、适合五一出游".to_string(),
                source_type: "manual".to_string(),
                source_id: None,
                confidence: 0.9,
                status: ProductKnowledgeStatus::Active,
                metadata: json!({}),
                created_at: now,
                updated_at: now,
            }],
            &[format!("product:{product_id}")],
        );

        assert!(summary.contains("五一咖啡组合"));
        assert!(summary.contains("核心卖点"));
        assert!(summary.contains("低糖、便携"));
        assert!(summary.contains(&format!("product:{product_id}")));
        assert!(summary.contains(&format!("product_knowledge:{knowledge_id}")));
    }

    #[test]
    fn query_terms_extract_commerce_keywords_from_chinese_input() {
        let terms = extract_query_terms("五一咖啡直播怎么提升转化率和ROI");

        assert!(terms.contains(&"五一".to_string()));
        assert!(terms.contains(&"咖啡".to_string()));
        assert!(terms.contains(&"直播".to_string()));
        assert!(terms.contains(&"转化".to_string()));
        assert!(terms.contains(&"roi".to_string()));
    }

    #[test]
    fn retrieval_summary_and_sources_reference_real_knowledge() {
        let product_id = Uuid::new_v4();
        let knowledge_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let product = Product {
            id: product_id,
            user_id: Uuid::new_v4(),
            name: "冷萃咖啡".to_string(),
            platform: Some("douyin".to_string()),
            external_id: None,
            category: Some("coffee".to_string()),
            brand: Some("Atlas".to_string()),
            status: ProductStatus::Active,
            description: None,
            price: Some("39".to_string()),
            currency: Some("CNY".to_string()),
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };
        let knowledge = ProductKnowledge {
            id: knowledge_id,
            user_id: product.user_id,
            product_id,
            title: Some("直播卖点".to_string()),
            content_type: "selling_point".to_string(),
            content: "适合五一出游，冷水 10 秒冲泡。".to_string(),
            source_type: "manual".to_string(),
            source_id: None,
            confidence: 0.92,
            status: ProductKnowledgeStatus::Active,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        let sources = build_retrieval_sources(
            std::slice::from_ref(&product),
            std::slice::from_ref(&knowledge),
        );
        let summary = build_retrieval_summary(&[product], &[knowledge], &["五一".to_string()]);

        assert_eq!(sources.len(), 2);
        assert!(summary.contains("Local commerce RAG context"));
        assert!(summary.contains("直播卖点"));
        assert!(summary.contains(&format!("product_knowledge:{knowledge_id}")));
    }

    #[test]
    fn degraded_reason_reflects_external_search_availability() {
        let unavailable = retrieval_degraded_reason(false);
        let available = retrieval_degraded_reason(true);

        assert!(unavailable.contains("not configured"));
        assert!(available.contains("configured"));
        assert!(available.contains("search skills may request live sources"));
    }
}

fn push_unique_uuid(items: &mut Vec<Uuid>, value: Uuid) {
    if !items.contains(&value) {
        items.push(value);
    }
}
