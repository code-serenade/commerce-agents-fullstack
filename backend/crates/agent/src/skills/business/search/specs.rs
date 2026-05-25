use serde_json::json;

use super::config::{MAX_RESULTS, category};
use crate::skills::{SkillInputField, SkillPriority, SkillSpec};

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

pub(super) fn search_trends_spec() -> SkillSpec {
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

pub(super) fn search_competitor_spec() -> SkillSpec {
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

pub(super) fn search_market_info_spec() -> SkillSpec {
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

pub(super) fn search_product_reviews_spec() -> SkillSpec {
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

pub(super) fn search_platform_policy_spec() -> SkillSpec {
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
                "query",
                "政策/规则关注点关键词，如价格合规、低价引流、虚假宣传",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "policy_type",
                "政策类型：algorithm/fee/promotion/content/material/general",
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

pub(super) fn search_industry_benchmarks_spec() -> SkillSpec {
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
                "关注指标：conversion/aov/nps/dsr/refund/roi/general",
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
    inputs.into_iter().chain([max_results_input()]).fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deferred")
            .with_tag("adapter")
            .with_tag("fresh_data"),
        SkillSpec::with_input,
    )
}

fn max_results_input() -> SkillInputField {
    SkillInputField::optional(
        "max_results",
        "最大可信来源数量，默认使用 external_search.default_max_results，允许 1-12",
        json!({"type": "integer", "minimum": 1, "maximum": MAX_RESULTS}),
    )
}
