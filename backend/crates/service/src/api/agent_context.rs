use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    product_knowledge::{
        ProductKnowledge, ProductKnowledgeFilter, ProductKnowledgeService, ProductKnowledgeStatus,
    },
    products::{Product, ProductService, ProductStatus},
};
use uuid::Uuid;

use crate::dto::agent_context::{
    AgentContextSource, AgentProductContextResponse, BuildAgentProductContextRequest,
};

const MAX_CONTEXT_PRODUCTS: usize = 5;
const MAX_CONTEXT_REFS: usize = 20;
const KNOWLEDGE_PER_PRODUCT_LIMIT: u64 = 6;
const MAX_CONTEXT_SUMMARY_CHARS: usize = 6000;
const MAX_KNOWLEDGE_CONTENT_CHARS: usize = 700;

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
    lines.push("Use these facts when relevant, cite product or knowledge titles, and do not invent missing product details.".to_string());

    for product in products {
        lines.push(format!(
            "Product source product:{} | name={} | platform={} | category={} | brand={} | price={} {} | status={} | description={}",
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
                "Knowledge source product_knowledge:{} | product={} | title={} | type={} | source={} | confidence={:.2} | content={}",
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

#[cfg(test)]
mod tests {
    use repo::table::{
        product_knowledge::{ProductKnowledge, ProductKnowledgeStatus},
        products::{Product, ProductStatus},
    };
    use serde_json::json;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::{build_context_summary, normalize_context_refs, normalize_product_ids};

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
}

fn push_unique_uuid(items: &mut Vec<Uuid>, value: Uuid) {
    if !items.contains(&value) {
        items.push(value);
    }
}
