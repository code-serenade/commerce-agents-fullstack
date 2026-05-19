use super::common::{field, spec};
use crate::skills::SkillSpec;

pub fn specs() -> Vec<SkillSpec> {
    vec![
        spec(
            "ops_pricing_strategy",
            "定价策略",
            "根据成本、竞品价格和平台，输出建议定价、利润率及竞品对比",
            vec![
                field("cost", "number", "单位成本（元）", true),
                field("competitor_prices", "array<number>", "竞品价格列表", true),
                field("platform", "string", "销售平台，默认淘宝", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
            ],
        ),
        spec(
            "ops_promo_planning",
            "促销策划",
            "根据商品、预算和平台，生成促销活动阶段、KPI和预算分配",
            vec![
                field("product", "string", "商品名称或类目", true),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
                field("budget", "number", "总预算（元）", true),
                field(
                    "platform",
                    "string",
                    "平台：淘宝/京东/拼多多/抖音/快手",
                    true,
                ),
            ],
        ),
        spec(
            "ops_channel_strategy",
            "渠道策略",
            "根据商品类型、预算和目标人群，输出渠道预算分配和内容计划",
            vec![
                field(
                    "product_type",
                    "string",
                    "商品类型：美妆/食品/数码/服饰/家居",
                    true,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
                field("budget", "number", "月预算（元）", true),
                field("target_audience", "string", "目标人群描述", false),
            ],
        ),
        spec(
            "ops_inventory_planning",
            "库存规划",
            "根据日均销量和供货周期，计算安全库存、补货点和建议订货量",
            vec![
                field("avg_daily_demand", "number", "日均销量（件）", true),
                field("lead_time_days", "integer", "供货周期（天）", true),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
                field("demand_std", "number", "需求标准差，默认日均需求20%", false),
                field(
                    "service_level",
                    "number",
                    "服务水平：0.90/0.95/0.99，默认0.95",
                    false,
                ),
            ],
        ),
        spec(
            "ops_listing_copy",
            "上架文案",
            "根据商品名称、卖点和平台，生成标题、卖点描述、关键词和问答",
            vec![
                field("product_name", "string", "商品名称", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
                field("features", "array<string>", "商品卖点列表", false),
                field("platform", "string", "上架平台，默认淘宝", false),
                field(
                    "product_id",
                    "integer",
                    "旧整数商品ID，仅用于兼容透传",
                    false,
                ),
                field("competitor_titles", "array<string>", "竞品标题列表", false),
            ],
        ),
        spec(
            "ops_assortment_planning",
            "选品规划",
            "根据类目和选品预算，输出商品组合、测品预算和风险控制",
            vec![
                field("category", "string", "商品类目", true),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
                field("budget", "number", "选品预算（元）", true),
            ],
        ),
        spec(
            "ops_execution_plan",
            "运营执行计划",
            "根据运营目标和阶段，生成30天行动计划、KPI体系和风险预案",
            vec![
                field("goal", "string", "运营目标", true),
                field("stage", "string", "阶段：冷启动/成长期/成熟期/衰退期", true),
                field(
                    "product_ref",
                    "string",
                    "新产品主推商品引用，如 product:{uuid}",
                    false,
                ),
                field("product_id", "integer", "旧整数主推商品ID，可选", false),
            ],
        ),
        spec(
            "ops_smart_pricing",
            "智能定价引擎",
            "基于价格弹性模型和竞品约束，计算最优利润定价区间和涨降价建议",
            vec![
                field("cost", "number", "单位成本（元）", true),
                field("current_price", "number", "当前售价（元）", true),
                field(
                    "current_monthly_sales",
                    "integer",
                    "当前月销量，默认500",
                    false,
                ),
                field("competitor_prices", "array<number>", "竞品价格列表", false),
                field("platform", "string", "平台，默认淘宝", false),
                field("elasticity", "number", "价格弹性系数，默认-1.5", false),
                field("stock_days", "integer", "当前库存天数", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
            ],
        ),
        spec(
            "ops_inventory_optimizer",
            "库存优化计算",
            "按SKU计算EOQ、安全库存、再订货点、库存健康和ABC分类",
            vec![
                field(
                    "skus",
                    "array<object>",
                    "SKU列表：{name, monthly_sales, cost, stock_qty}",
                    false,
                ),
                field("ordering_cost", "number", "单次订货成本，默认200", false),
                field("holding_rate", "number", "年库存持有成本率%，默认25", false),
                field("lead_time_days", "integer", "供应商交货周期，默认14", false),
                field(
                    "service_level",
                    "string",
                    "服务水平：90%/95%/98%/99%，默认95%",
                    false,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
            ],
        ),
        spec(
            "ops_ad_fatigue_detector",
            "广告疲劳检测",
            "检测CTR衰减、曝光频次和ROAS健康度，输出疲劳评分与预算重分配",
            vec![
                field(
                    "channels",
                    "array<object>",
                    "渠道列表：{name, ctr_baseline, ctr_recent, impressions_per_user, budget, \
                     roas}",
                    false,
                ),
                field("total_budget", "number", "总广告预算", false),
                field("platform", "string", "平台，默认淘宝", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
            ],
        ),
        spec(
            "ops_abc_xyz_classifier",
            "ABC-XYZ库存分类",
            "ABC按GMV价值、XYZ按需求变异系数CV生成9格库存策略矩阵",
            vec![
                field(
                    "skus",
                    "array<object>",
                    "SKU列表：{sku_id, gmv, sales_history:[...]}",
                    false,
                ),
                field("platform", "string", "平台，默认淘宝", false),
                field("top_n", "integer", "展示前N个高价值SKU，默认20", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用，如 product:{uuid}",
                    false,
                ),
            ],
        ),
    ]
}
