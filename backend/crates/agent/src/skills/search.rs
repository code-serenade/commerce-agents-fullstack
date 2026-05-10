use async_trait::async_trait;
use serde_json::{Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_i64_param, optional_string_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("search".to_owned())
}

const SEARCH_ADAPTER_REQUIRED_CONFIG: &str = "search.provider / search endpoint not configured";
const SEARCH_DEFERRED_REASON: &str = "外部搜索 adapter 尚未接入，不能伪造实时来源";
const SEARCH_DEGRADED_REASON: &str =
    "Search adapter is not configured; planned query is returned for traceability.";
const DEFAULT_MAX_RESULTS: i64 = 6;

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
        }
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
            "max_results": plan.max_results
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
        "degraded": true,
        "degraded_reason": SEARCH_DEGRADED_REASON,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "_search_engine"]
    }))
    .with_summary("外部搜索 adapter 未接入，已返回结构化降级契约")
    .with_metadata("adapter_status", json!("adapter_required"))
    .with_metadata("adapter_kind", json!("search"))
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

            async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
                object_params(&params)?;
                let required_value = string_param(&params, $required)?;
                let plan = $builder(&params, &required_value)?;
                Ok(deferred_search_output(&self.spec.id, required_value, plan))
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
