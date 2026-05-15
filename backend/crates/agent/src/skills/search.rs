use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use toolcraft_request::{HeaderMap, Request};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_bool_param, optional_i64_param, optional_string_param,
    string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("search".to_owned())
}

const SEARCH_ADAPTER_REQUIRED_CONFIG: &str = "search.provider / search endpoint not configured";
const SEARCH_DEFERRED_REASON: &str = "外部搜索 adapter 尚未接入，不能伪造实时来源";
const SEARCH_DEGRADED_REASON: &str =
    "Search adapter is not configured; planned query is returned for traceability.";
const SEARCH_PROVIDER_EMPTY_REASON: &str =
    "External search provider returned no normalized sources; live sources were not accepted.";
const VALID_TOPICS: &[&str] = &["general", "news", "finance"];
const MAX_DAYS: i64 = 365;
const MAX_RESULTS: i64 = 12;
const DEFAULT_MAX_RESULTS: i64 = 6;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SearchAdapterConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub search_endpoint: Option<String>,
    pub api_key: Option<String>,
    pub timeout_sec: Option<u64>,
    pub default_topic: Option<String>,
    pub default_days: Option<u64>,
    pub default_max_results: Option<u64>,
}

impl Default for SearchAdapterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            base_url: None,
            search_endpoint: None,
            api_key: None,
            timeout_sec: Some(10),
            default_topic: Some("general".to_string()),
            default_days: Some(30),
            default_max_results: Some(DEFAULT_MAX_RESULTS as u64),
        }
    }
}

impl SearchAdapterConfig {
    pub fn is_configured(&self) -> bool {
        self.enabled
            && self
                .base_url
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .search_endpoint
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }

    pub fn provider_name(&self) -> String {
        self.provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("external_search")
            .to_string()
    }
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        search_trends_spec(),
        search_competitor_spec(),
        search_market_info_spec(),
        search_product_reviews_spec(),
        search_platform_policy_spec(),
        search_industry_benchmarks_spec(),
    ]
}

pub fn search_trends_spec() -> SkillSpec {
    search_spec(
        "search_trends",
        "市场趋势搜索",
        "实时搜索热卖商品、爆款话题、消费趋势、流行风格",
        vec![
            SkillInputField::required(
                "query",
                "搜索关键词，如商品类目、品类名称、热门话题",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台（淘宝/抖音/小红书/京东/拼多多，可选）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "days",
                "搜索最近N天内信息（默认30天）",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub fn search_competitor_spec() -> SkillSpec {
    search_spec(
        "search_competitor",
        "竞品信息搜索",
        "实时搜索竞争对手商品信息、定价策略、促销活动、用户评价和差异化卖点",
        vec![
            SkillInputField::required(
                "query",
                "竞品关键词，如商品名称、品牌名、品类名",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台（淘宝/京东/拼多多/抖音，可选）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "focus",
                "分析重点：price/promotion/reviews/features",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn search_market_info_spec() -> SkillSpec {
    search_spec(
        "search_market_info",
        "市场行情搜索",
        "实时搜索行业市场报告、政策法规变化、平台规则更新、行业新闻和市场数据",
        vec![
            SkillInputField::required(
                "query",
                "行业/话题关键词，如行业名、政策名、市场主题",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "topic",
                "搜索类型：general/news/finance",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "days",
                "搜索最近N天内信息（默认7天）",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub fn search_product_reviews_spec() -> SkillSpec {
    search_spec(
        "search_product_reviews",
        "用户评价搜索",
        "实时搜索指定商品或品牌的真实用户评价、口碑反馈、好差评分布",
        vec![
            SkillInputField::required(
                "query",
                "商品名称、品牌名或品类关键词",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台（淘宝/京东/拼多多/抖音，可选）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "focus",
                "关注点：positive/negative/overall",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn search_platform_policy_spec() -> SkillSpec {
    search_spec(
        "search_platform_policy",
        "平台政策搜索",
        "实时搜索电商平台最新规则变化、算法调整、流量政策、佣金费率、大促规则",
        vec![
            SkillInputField::required(
                "platform",
                "平台名称（淘宝/京东/拼多多/抖音/小红书等）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "policy_type",
                "政策类型：algorithm/fee/promotion/content/general",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "days",
                "搜索最近N天内信息（默认30天）",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub fn search_industry_benchmarks_spec() -> SkillSpec {
    search_spec(
        "search_industry_benchmarks",
        "行业基准搜索",
        "实时搜索行业转化率、客单价、NPS、DSR、退款率、广告ROI等KPI基准数据",
        vec![
            SkillInputField::required(
                "industry",
                "行业类目（如：美妆/服装/3C/食品/家居等）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "metric",
                "关注指标：conversion/aov/nps/refund/roi/general",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台（可选，不填则返回全平台基准）",
                json!({"type": "string"}),
            ),
        ],
    )
}

fn search_spec(id: &str, name: &str, description: &str, inputs: Vec<SkillInputField>) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deferred")
            .with_tag("adapter")
            .with_tag("fresh_data"),
        SkillSpec::with_input,
    )
}

#[derive(Debug, Clone)]
struct SearchPlan {
    raw: String,
    planned: String,
    topic: String,
    days: i64,
    max_results: i64,
    bypass_cache: bool,
    product_ref: Option<String>,
    product_name: Option<String>,
    product_category: Option<String>,
}

impl SearchPlan {
    fn new(
        raw: impl Into<String>,
        planned: impl Into<String>,
        topic: impl Into<String>,
        days: i64,
    ) -> Self {
        Self {
            raw: raw.into(),
            planned: planned.into(),
            topic: topic.into(),
            days: days.max(1),
            max_results: DEFAULT_MAX_RESULTS,
            bypass_cache: false,
            product_ref: None,
            product_name: None,
            product_category: None,
        }
    }

    fn with_bypass_cache(mut self, bypass_cache: bool) -> Self {
        self.bypass_cache = bypass_cache;
        self
    }

    fn with_product_context(
        mut self,
        product_ref: Option<String>,
        product_name: Option<String>,
        product_category: Option<String>,
    ) -> Self {
        self.product_ref = product_ref;
        self.product_name = product_name;
        self.product_category = product_category;
        self
    }

    fn with_adapter_defaults(
        mut self,
        adapter: &SearchAdapterConfig,
    ) -> Result<Self, crate::skills::SkillError> {
        self.topic = normalize_topic(&self.topic, adapter.default_topic.as_deref())?;
        self.days = self.days.clamp(1, MAX_DAYS);
        self.max_results = adapter
            .default_max_results
            .unwrap_or(DEFAULT_MAX_RESULTS as u64)
            .clamp(1, MAX_RESULTS as u64) as i64;
        Ok(self)
    }
}

fn deferred_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
) -> SkillOutcome {
    SkillOutcome::new(json!({
        "status": "deferred",
        "adapter_status": "adapter_required",
        "adapter": {
            "kind": "search",
            "provider": null,
            "configured": false,
            "required_config": SEARCH_ADAPTER_REQUIRED_CONFIG
        },
        "deferred_reason": SEARCH_DEFERRED_REASON,
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "data": {
            "items": []
        },
        "sources": [],
        "provenance": {
            "source_count": 0,
            "domain_count": 0,
            "domains": [],
            "published_count": 0,
            "latest_published_at": null
        },
        "confidence": 0.0,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": null
        },
        "evidence": build_search_evidence(&[]),
        "degraded": true,
        "degraded_reason": SEARCH_DEGRADED_REASON,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary("外部搜索 adapter 未接入，已返回结构化降级契约")
    .with_metadata("adapter_status", json!("adapter_required"))
    .with_metadata("adapter_kind", json!("search"))
}

fn degraded_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
    adapter: &SearchAdapterConfig,
    reason: impl Into<String>,
) -> SkillOutcome {
    let reason = reason.into();
    SkillOutcome::new(json!({
        "status": "deferred",
        "adapter_status": "degraded",
        "adapter": {
            "kind": "search",
            "provider": adapter.provider_name(),
            "configured": adapter.is_configured(),
            "required_config": if adapter.is_configured() { Value::Null } else { json!(SEARCH_ADAPTER_REQUIRED_CONFIG) }
        },
        "deferred_reason": "外部搜索 adapter 已配置但本次未拿到可采信来源，不能伪造实时来源",
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "engine": adapter.provider_name(),
        "data": {
            "items": []
        },
        "sources": [],
        "provenance": {
            "source_count": 0,
            "domain_count": 0,
            "domains": [],
            "published_count": 0,
            "latest_published_at": null
        },
        "confidence": 0.0,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": utc_now_rfc3339()
        },
        "evidence": build_search_evidence(&[]),
        "notice": "未获取到可靠公开来源，当前不能伪造实时搜索结果。",
        "degraded": true,
        "degraded_reason": reason,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary("外部搜索 adapter 未返回可采信来源，已按降级结果处理")
    .with_metadata("adapter_status", json!("degraded"))
    .with_metadata("adapter_kind", json!("search"))
    .with_metadata("provider", json!(adapter.provider_name()))
}

fn live_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
    adapter: &SearchAdapterConfig,
    payload: &Value,
) -> SkillOutcome {
    let sources = normalize_sources(payload);
    if sources.is_empty() {
        return degraded_search_output(
            skill_id,
            required_input,
            plan,
            adapter,
            SEARCH_PROVIDER_EMPTY_REASON,
        );
    }

    let provenance = build_provenance(&sources);
    let evidence = build_search_evidence(&sources);
    let confidence = payload
        .get("confidence")
        .and_then(Value::as_f64)
        .map(|value| (value * 1000.0).round() / 1000.0)
        .unwrap_or_else(|| estimate_confidence(&sources));
    let engine = payload
        .get("engine")
        .or_else(|| payload.get("_search_engine"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| adapter.provider_name());
    let requested_at = payload
        .get("requested_at")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(utc_now_rfc3339);
    let source_count = provenance
        .get("source_count")
        .and_then(Value::as_u64)
        .unwrap_or(sources.len() as u64);

    SkillOutcome::new(json!({
        "status": "live",
        "adapter_status": "live",
        "adapter": {
            "kind": "search",
            "provider": adapter.provider_name(),
            "configured": true,
            "required_config": null
        },
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "key": payload.get("key").and_then(Value::as_str).unwrap_or("search"),
        "engine": engine,
        "data": {
            "items": sources
        },
        "sources": sources,
        "provenance": provenance,
        "confidence": confidence,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": requested_at
        },
        "evidence": evidence,
        "degraded": false,
        "degraded_reason": null,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary(format!("外部搜索已返回 {source_count} 个可采信来源"))
    .with_metadata("adapter_status", json!("live"))
    .with_metadata("adapter_kind", json!("search"))
    .with_metadata("provider", json!(adapter.provider_name()))
    .with_metadata("source_count", json!(source_count))
}

async fn query_provider(adapter: &SearchAdapterConfig, plan: &SearchPlan) -> Result<Value, String> {
    let base_url = adapter
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SEARCH_ADAPTER_REQUIRED_CONFIG.to_string())?;
    let endpoint = adapter
        .search_endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SEARCH_ADAPTER_REQUIRED_CONFIG.to_string())?;
    let mut client = Request::with_timeout(adapter.timeout_sec.unwrap_or(10).max(1))
        .map_err(|err| format!("init search request client failed: {err}"))?;
    client
        .set_base_url(base_url)
        .map_err(|err| format!("invalid search base_url: {err}"))?;

    let query = vec![
        ("q".to_string(), plan.planned.clone()),
        ("topic".to_string(), plan.topic.clone()),
        ("days".to_string(), plan.days.to_string()),
        ("max_results".to_string(), plan.max_results.to_string()),
        ("bypass_cache".to_string(), plan.bypass_cache.to_string()),
    ];
    let response = client
        .get(endpoint, Some(query), build_provider_headers(adapter)?)
        .await
        .map_err(|err| format!("external search provider request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        return Err(format!(
            "external search provider returned status {}: {}",
            status.as_u16(),
            compact_text(&body, 240)
        ));
    }
    response
        .json::<Value>()
        .await
        .map_err(|err| format!("decode external search provider response failed: {err}"))
}

fn normalize_sources(payload: &Value) -> Vec<Value> {
    let Some(items) = payload
        .pointer("/data/items")
        .and_then(Value::as_array)
        .or_else(|| payload.get("sources").and_then(Value::as_array))
        .or_else(|| payload.get("results").and_then(Value::as_array))
    else {
        return Vec::new();
    };

    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();
    for item in items {
        let rank = sources.len() as u64 + 1;
        let Some(source) = normalize_source(rank, item) else {
            continue;
        };
        let Some(key) = normalized_source_key(&source) else {
            continue;
        };
        if seen.insert(key) {
            sources.push(source);
        }
        if sources.len() >= MAX_RESULTS as usize {
            break;
        }
    }
    sources
}

fn normalize_source(rank: u64, item: &Value) -> Option<Value> {
    let title = string_field(item, &["title", "name"])
        .map(|value| compact_text(&value, 180))
        .unwrap_or_else(|| "未命名来源".to_string());
    let url = string_field(item, &["url", "link"]).unwrap_or_default();
    if title.trim().is_empty() && url.trim().is_empty() {
        return None;
    }
    let snippet = string_field(item, &["snippet", "body", "summary", "content"])
        .map(|value| compact_text(&value, 320))
        .unwrap_or_default();
    let published_at = string_field(item, &["published_at", "publishedAt", "pubDate"])
        .map(|value| compact_text(&value, 64))
        .filter(|value| !value.is_empty());
    let domain = extract_domain(&url);

    Some(json!({
        "rank": rank,
        "title": title,
        "snippet": snippet,
        "url": compact_text(&url, 400),
        "domain": domain,
        "published_at": published_at
    }))
}

fn normalized_source_key(source: &Value) -> Option<String> {
    let url = source
        .get("url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let title = source
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    url.or(title)
        .map(|value| value.trim_end_matches('/').to_ascii_lowercase())
}

fn build_provenance(sources: &[Value]) -> Value {
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(12)
        .collect::<Vec<_>>();
    let published = sources
        .iter()
        .filter_map(|item| item.get("published_at").and_then(Value::as_str))
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let latest_published_at = published.iter().max().cloned();

    json!({
        "source_count": sources.len(),
        "domain_count": domains.len(),
        "domains": domains,
        "published_count": published.len(),
        "latest_published_at": latest_published_at
    })
}

fn build_search_evidence(sources: &[Value]) -> Value {
    if sources.is_empty() {
        return json!({
            "summary": "没有可采信实时来源；当前结果只能作为降级状态，不可当作联网证据。",
            "source_titles": [],
            "source_domains": [],
            "llm_context": "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）"
        });
    }

    let titles = sources
        .iter()
        .filter_map(|item| item.get("title").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 120))
        .take(5)
        .collect::<Vec<_>>();
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(8)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let summary = format!(
        "共 {} 条实时来源，覆盖 {} 个域名；回答应优先引用这些来源，并注明时效性和置信度。",
        sources.len(),
        domains.len()
    );

    json!({
        "summary": summary,
        "source_titles": titles,
        "source_domains": domains,
        "llm_context": format_sources_for_llm(sources)
    })
}

fn format_sources_for_llm(sources: &[Value]) -> String {
    if sources.is_empty() {
        return "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）".to_string();
    }

    let mut lines = vec![format!(
        "（共{}条实时来源，优先从中提取具体数字、结论和出处；若来源不足，明确说明数据不足。）",
        sources.len().min(5)
    )];
    for (index, source) in sources.iter().take(5).enumerate() {
        let title = source
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("未命名来源");
        let snippet = source
            .get("snippet")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        let url = source
            .get("url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        let published_at = source
            .get("published_at")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());

        lines.push(format!("\n[{}] {}", index + 1, compact_text(title, 120)));
        if let Some(value) = published_at {
            lines.push(format!("发布时间：{}", compact_text(value, 64)));
        }
        if !snippet.is_empty() {
            lines.push(compact_text(snippet, 320));
        }
        if !url.is_empty() {
            lines.push(format!("来源：{}", compact_text(url, 300)));
        }
    }
    lines.push(
        "\n注：遇到实时来源与通用经验不一致时，以实时来源为准，并说明来源数量、时效性和不确定性。"
            .to_string(),
    );
    lines.join("\n")
}

fn estimate_confidence(sources: &[Value]) -> f64 {
    if sources.is_empty() {
        return 0.0;
    }
    let source_score = (sources.len() as f64 / DEFAULT_MAX_RESULTS as f64).min(1.0) * 0.55;
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>();
    let diversity_score = (domains.len() as f64 / sources.len() as f64).min(1.0) * 0.25;
    let freshness_score = if sources
        .iter()
        .any(|item| item.get("published_at").and_then(Value::as_str).is_some())
    {
        0.2
    } else {
        0.08
    };
    ((source_score + diversity_score + freshness_score).min(1.0) * 1000.0).round() / 1000.0
}

fn build_provider_headers(adapter: &SearchAdapterConfig) -> Result<Option<HeaderMap>, String> {
    let Some(api_key) = adapter
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let mut headers = HeaderMap::new();
    headers
        .insert("Authorization", format!("Bearer {api_key}"))
        .map_err(|err| format!("invalid search auth header: {err}"))?;
    Ok(Some(headers))
}

fn normalize_topic(
    topic: &str,
    fallback: Option<&str>,
) -> Result<String, crate::skills::SkillError> {
    let normalized = topic.trim().to_ascii_lowercase();
    let normalized = if normalized.is_empty() {
        fallback
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("general")
            .to_ascii_lowercase()
    } else {
        normalized
    };
    if !VALID_TOPICS.contains(&normalized.as_str()) {
        return Err(crate::skills::SkillError::invalid_parameter(
            "topic",
            "general/news/finance",
        ));
    }
    Ok(normalized)
}

fn search_bypass_cache(params: &Value) -> Result<bool, crate::skills::SkillError> {
    Ok(optional_bool_param(params, "bypass_cache")?
        .or(optional_bool_param(params, "bypassCache")?)
        .or(optional_bool_param(params, "_needs_fresh_data")?)
        .unwrap_or(false))
}

fn optional_search_string_param(
    params: &Value,
    field: &str,
) -> Result<Option<String>, crate::skills::SkillError> {
    optional_string_param(params, field)
}

fn string_field(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_domain(url: &str) -> String {
    let text = url.trim();
    if text.is_empty() {
        return String::new();
    }
    let without_scheme = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .unwrap_or(text);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches("www.")
        .to_ascii_lowercase()
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect()
}

fn utc_now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

macro_rules! deferred_search_skill {
    ($type_name:ident, $spec_fn:ident, $required:literal, $builder:expr) => {
        pub struct $type_name {
            spec: SkillSpec,
        }

        impl $type_name {
            pub fn new() -> Self {
                Self { spec: $spec_fn() }
            }

            pub async fn run_with_adapter(
                &self,
                params: Value,
                _context: SkillContext,
                adapter: &SearchAdapterConfig,
            ) -> SkillResult {
                object_params(&params)?;
                let required_value = string_param(&params, $required)?;
                let bypass_cache = search_bypass_cache(&params)?;
                let product_ref = optional_search_string_param(&params, "product_ref")?;
                let product_name = optional_search_string_param(&params, "product_name")?;
                let product_category = optional_search_string_param(&params, "product_category")?;
                let plan = $builder(&params, &required_value)?
                    .with_bypass_cache(bypass_cache)
                    .with_product_context(product_ref, product_name, product_category)
                    .with_adapter_defaults(adapter)?;
                if !adapter.is_configured() {
                    return Ok(deferred_search_output(&self.spec.id, required_value, plan));
                }

                let payload = match query_provider(adapter, &plan).await {
                    Ok(payload) => payload,
                    Err(reason) => {
                        return Ok(degraded_search_output(
                            &self.spec.id,
                            required_value,
                            plan,
                            adapter,
                            reason,
                        ));
                    }
                };
                Ok(live_search_output(
                    &self.spec.id,
                    required_value,
                    plan,
                    adapter,
                    &payload,
                ))
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[async_trait]
        impl Skill for $type_name {
            fn spec(&self) -> &SkillSpec {
                &self.spec
            }

            async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
                self.run_with_adapter(params, context, &SearchAdapterConfig::default())
                    .await
            }
        }
    };
}

deferred_search_skill!(
    SearchTrends,
    search_trends_spec,
    "query",
    build_trends_query
);
deferred_search_skill!(
    SearchCompetitor,
    search_competitor_spec,
    "query",
    build_competitor_query
);
deferred_search_skill!(
    SearchMarketInfo,
    search_market_info_spec,
    "query",
    build_market_info_query
);
deferred_search_skill!(
    SearchProductReviews,
    search_product_reviews_spec,
    "query",
    build_reviews_query
);
deferred_search_skill!(
    SearchPlatformPolicy,
    search_platform_policy_spec,
    "platform",
    build_policy_query
);
deferred_search_skill!(
    SearchIndustryBenchmarks,
    search_industry_benchmarks_spec,
    "industry",
    build_benchmark_query
);

fn build_trends_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_else(|| "电商".to_owned());
    let days = optional_i64_param(params, "days")?.unwrap_or(30);
    let planned = format!("{query} {platform} 热销 趋势 爆款 2026 最近{days}天");
    Ok(SearchPlan::new(query, planned, "general", days))
}

fn build_competitor_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_default();
    let focus = optional_string_param(params, "focus")?.unwrap_or_default();
    let focus_text = match focus.as_str() {
        "price" => "价格 定价 价位 售价",
        "promotion" => "促销 活动 优惠 折扣 限时",
        "reviews" => "用户评价 口碑 差评 真实体验",
        "features" => "卖点 功能特点 优势 核心参数",
        _ => "价格 卖点 评价 优惠 竞品对比",
    };
    let planned = format!("{query} {platform} {focus_text} 2026");
    Ok(SearchPlan::new(query, planned, "general", 60))
}

fn build_market_info_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let days = optional_i64_param(params, "days")?.unwrap_or(7);
    let topic = optional_string_param(params, "topic")?.unwrap_or_else(|| "news".to_owned());
    let recency = if days <= 7 {
        "最新".to_owned()
    } else {
        format!("{days}天内")
    };
    let planned = format!("{query} {recency} 行业动态 市场分析 2026");
    Ok(SearchPlan::new(query, planned, topic, days))
}

fn build_reviews_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_default();
    let focus = optional_string_param(params, "focus")?.unwrap_or_else(|| "overall".to_owned());
    let focus_text = match focus.as_str() {
        "positive" => "好评 亮点 优点 推荐 满意 值得买",
        "negative" => "差评 缺点 投诉 退货 踩雷 槽点",
        _ => "用户评价 口碑 真实反馈 购买体验 评测",
    };
    let planned = format!("{query} {platform} {focus_text} 2026");
    Ok(SearchPlan::new(query, planned, "general", 90))
}

fn build_policy_query(
    params: &Value,
    platform: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let policy_type =
        optional_string_param(params, "policy_type")?.unwrap_or_else(|| "general".to_owned());
    let days = optional_i64_param(params, "days")?.unwrap_or(30);
    let type_text = match policy_type.as_str() {
        "algorithm" => "搜索算法 流量规则 排名机制 权重调整",
        "fee" => "佣金 费率 扣点 服务费 收费标准",
        "promotion" => "大促规则 活动报名 双11 618 商家政策",
        "content" => "内容规范 违禁词 审核标准 发布规定 违规处罚",
        _ => "规则变化 政策更新 新规 公告",
    };
    let planned = format!("{platform} {type_text} 最新 2026");
    Ok(SearchPlan::new(platform, planned, "news", days))
}

fn build_benchmark_query(
    params: &Value,
    industry: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_else(|| "电商".to_owned());
    let metric = optional_string_param(params, "metric")?.unwrap_or_else(|| "general".to_owned());
    let metric_text = match metric.as_str() {
        "conversion" => "转化率 成交转化 CVR 行业均值 基准",
        "aov" => "客单价 平均订单金额 消费水平 均值",
        "nps" => "NPS 净推荐值 客户满意度 行业标准 平均分",
        "refund" => "退款率 退货率 售后率 行业水平 均值",
        "roi" => "广告ROI 投产比 ROAS 营销效率 基准",
        _ => "行业数据 KPI基准 平均水平 运营指标 数据报告",
    };
    let planned = format!("{industry} {platform} {metric_text} 2025 2026");
    Ok(SearchPlan::new(industry, planned, "general", 180))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_provider_sources_into_deduped_evidence() {
        let payload = json!({
            "data": {
                "items": [
                    {
                        "title": "咖啡消费趋势",
                        "snippet": "冷萃咖啡增长",
                        "url": "https://www.example.com/a",
                        "published_at": "2026-05-01T00:00:00Z"
                    },
                    {
                        "title": "咖啡消费趋势重复",
                        "summary": "重复 URL 应被去掉",
                        "url": "https://www.example.com/a/"
                    },
                    {
                        "title": "咖啡行业报告",
                        "body": "即饮咖啡继续增长",
                        "url": "https://report.example.cn/b"
                    }
                ]
            }
        });

        let sources = normalize_sources(&payload);
        let evidence = build_search_evidence(&sources);

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].get("rank").and_then(Value::as_u64), Some(1));
        assert_eq!(sources[1].get("rank").and_then(Value::as_u64), Some(2));
        assert!(
            evidence
                .get("summary")
                .and_then(Value::as_str)
                .is_some_and(|summary| summary.contains("2 条实时来源"))
        );
        assert!(
            evidence
                .get("llm_context")
                .and_then(Value::as_str)
                .is_some_and(|context| context.contains("咖啡消费趋势")
                    && context.contains("https://www.example.com/a"))
        );
    }

    #[tokio::test]
    async fn search_skill_preserves_fresh_data_bypass_in_degraded_output() {
        let adapter = SearchAdapterConfig {
            enabled: false,
            ..SearchAdapterConfig::default()
        };
        let outcome = SearchTrends::new()
            .run_with_adapter(
                json!({
                    "query": "咖啡",
                    "_needs_fresh_data": true
                }),
                SkillContext::default(),
                &adapter,
            )
            .await
            .expect("search should return structured deferred output");

        assert_eq!(
            outcome.output.pointer("/query/bypass_cache"),
            Some(&json!(true))
        );
        assert_eq!(
            outcome.output.pointer("/evidence/source_titles"),
            Some(&json!([]))
        );
        assert!(
            outcome
                .output
                .pointer("/evidence/llm_context")
                .and_then(Value::as_str)
                .is_some_and(|context| context.contains("没有可采信实时来源"))
        );
    }

    #[tokio::test]
    async fn search_skill_echoes_product_context_in_degraded_output() {
        let adapter = SearchAdapterConfig {
            enabled: false,
            ..SearchAdapterConfig::default()
        };
        let outcome = SearchCompetitor::new()
            .run_with_adapter(
                json!({
                    "query": "五一咖啡组合 咖啡饮品",
                    "platform": "抖音",
                    "focus": "price",
                    "product_ref": "product:11111111-1111-1111-1111-111111111111",
                    "product_name": "五一咖啡组合",
                    "product_category": "咖啡饮品"
                }),
                SkillContext::default(),
                &adapter,
            )
            .await
            .expect("search should return structured deferred output");

        assert_eq!(
            outcome.output.pointer("/query/product_ref"),
            Some(&json!("product:11111111-1111-1111-1111-111111111111"))
        );
        assert_eq!(
            outcome.output.pointer("/query/product_name"),
            Some(&json!("五一咖啡组合"))
        );
        assert_eq!(
            outcome.output.pointer("/query/product_category"),
            Some(&json!("咖啡饮品"))
        );
        assert_eq!(outcome.output.pointer("/status"), Some(&json!("deferred")));
    }
}
