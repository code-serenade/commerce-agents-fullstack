use async_trait::async_trait;
use serde_json::Map;
use serde_json::{Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec,
};

fn category() -> SkillCategory {
    SkillCategory::Data
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        spec(
            "query_store_metrics",
            "查询店铺指标",
            "查询店铺经营指标；支持本轮显式指标输入，未接数据源时明确降级",
            SkillPriority::High,
            vec![
                field("days", "integer", "查询最近N天，默认30", false),
                field(
                    "platform",
                    "string",
                    "平台筛选：taobao/jd/pdd/douyin，留空查全部",
                    false,
                ),
                field(
                    "include_anomalies",
                    "boolean",
                    "是否同时检测异常，默认true",
                    false,
                ),
                field("gmv", "number", "本期GMV", false),
                field("previous_gmv", "number", "上期GMV，用于环比", false),
                field("orders", "integer", "本期订单数", false),
                field("previous_orders", "integer", "上期订单数，用于环比", false),
                field("visitors", "integer", "本期访客数UV", false),
                field(
                    "previous_visitors",
                    "integer",
                    "上期访客数UV，用于环比",
                    false,
                ),
                field("clicks", "integer", "本期点击数", false),
                field("ad_spend", "number", "广告花费", false),
                field("refund_amount", "number", "退款金额", false),
                field("gross_profit", "number", "毛利额", false),
                field(
                    "platforms",
                    "array<object>",
                    "平台指标列表：{platform,gmv,orders,uv,clicks}",
                    false,
                ),
            ],
            &["deterministic"],
        ),
        spec(
            "query_product_metrics",
            "查询商品指标",
            "查询单个商品或SKU经营指标；支持本轮显式指标输入，未接数据源时明确降级",
            SkillPriority::High,
            vec![
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品或显式输入",
                    false,
                ),
                field(
                    "product_id",
                    "string",
                    "商品ID；只作为本轮显式筛选条件",
                    false,
                ),
                field(
                    "product_name",
                    "string",
                    "商品名称；只作为本轮显式筛选条件",
                    false,
                ),
                field("sku_id", "string", "SKU ID；只作为本轮显式筛选条件", false),
                field("days", "integer", "查询最近N天，默认30", false),
                field("platform", "string", "平台筛选，留空查全部", false),
                field("gmv", "number", "商品GMV", false),
                field("previous_gmv", "number", "上期商品GMV，用于环比", false),
                field("orders", "integer", "商品订单数", false),
                field(
                    "previous_orders",
                    "integer",
                    "上期商品订单数，用于环比",
                    false,
                ),
                field("visitors", "integer", "商品访客数UV", false),
                field("clicks", "integer", "商品点击数", false),
                field("ad_spend", "number", "商品广告花费", false),
                field("refund_amount", "number", "商品退款金额", false),
                field("gross_profit", "number", "商品毛利额", false),
                field("inventory", "integer", "当前库存", false),
                field("stockout_count", "integer", "缺货次数", false),
            ],
            &["deterministic"],
        ),
        spec(
            "query_campaign_metrics",
            "查询活动指标",
            "查询营销活动或投放活动指标；支持本轮显式指标输入，未接数据源时明确降级",
            SkillPriority::High,
            vec![
                field(
                    "campaign_id",
                    "string",
                    "活动ID；只作为本轮显式筛选条件",
                    false,
                ),
                field(
                    "campaign_name",
                    "string",
                    "活动名称；只作为本轮显式筛选条件",
                    false,
                ),
                field(
                    "campaign_type",
                    "string",
                    "活动类型：promotion/ad/live/launch等",
                    false,
                ),
                field("days", "integer", "查询最近N天，默认30", false),
                field("platform", "string", "平台筛选，留空查全部", false),
                field("budget", "number", "活动预算", false),
                field("spend", "number", "活动实际花费", false),
                field("ad_spend", "number", "广告花费", false),
                field("gmv", "number", "活动GMV", false),
                field("previous_gmv", "number", "上期活动GMV，用于环比", false),
                field("orders", "integer", "活动订单数", false),
                field("visitors", "integer", "活动访客数UV", false),
                field("clicks", "integer", "活动点击数", false),
                field("impressions", "integer", "活动曝光量", false),
                field("coupon_cost", "number", "优惠券/让利成本", false),
                field("refund_amount", "number", "退款金额", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_funnel_analysis",
            "漏斗分析",
            "分析电商转化漏斗各阶段转化率并定位流失环节",
            SkillPriority::Critical,
            vec![
                field("impressions", "integer", "曝光量", false),
                field("clicks", "integer", "点击量", false),
                field("add_to_cart", "integer", "加购量", false),
                field("orders", "integer", "下单量", false),
                field("payments", "integer", "付款量", false),
                field("platform", "string", "平台", false),
                field("days", "integer", "查询天数，默认30", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_anomaly_diagnosis",
            "异常诊断",
            "基于Z-Score检测指标时序异常点",
            SkillPriority::Critical,
            vec![
                field("metric_name", "string", "指标名称，默认未知指标", false),
                field("days", "integer", "分析天数，默认14", false),
                field("values", "array<number>", "手动提供的时序数据", false),
                field("threshold", "number", "Z-Score阈值，默认2.0", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_trend_forecast",
            "趋势预测",
            "基于线性趋势预测GMV、UV、订单等核心指标",
            SkillPriority::Critical,
            vec![
                field(
                    "metric_name",
                    "string",
                    "指标：gmv/orders/uv/conversion_rate，默认gmv",
                    false,
                ),
                field("days", "integer", "历史天数，默认30", false),
                field("forecast_periods", "integer", "预测期数，默认7天", false),
                field("platform", "string", "平台筛选", false),
                field("values", "array<number>", "手动提供的历史序列", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_dashboard",
            "数据看板",
            "生成店铺核心经营指标看板，支持手工指标输入和未接数据源时的明确降级说明",
            SkillPriority::High,
            vec![
                field("days", "integer", "统计天数，默认30", false),
                field("platform", "string", "平台筛选，留空查全部", false),
                field("gmv", "number", "本期GMV", false),
                field("previous_gmv", "number", "上期GMV，用于环比", false),
                field("orders", "integer", "本期订单数", false),
                field("previous_orders", "integer", "上期订单数，用于环比", false),
                field("visitors", "integer", "本期访客数UV", false),
                field(
                    "previous_visitors",
                    "integer",
                    "上期访客数UV，用于环比",
                    false,
                ),
                field("clicks", "integer", "本期点击数", false),
                field("ad_spend", "number", "广告花费", false),
                field("refund_amount", "number", "退款金额", false),
                field("gross_profit", "number", "毛利额", false),
                field("target_gmv", "number", "目标GMV，用于完成率", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_customer_segmentation",
            "RFM客户分层",
            "基于RFM模型输出客户分群、规模和运营策略",
            SkillPriority::Critical,
            vec![
                field("total_customers", "integer", "总客户数", false),
                field("avg_order_value", "number", "平均客单价", false),
                field("avg_orders_per_customer", "number", "人均订单数", false),
                field("days", "integer", "数据范围，默认90天", false),
                field("recency_days", "number", "平均近期购买间隔天数", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_competitor_analysis",
            "竞品分析",
            "输入竞品信息，输出竞品对比矩阵和策略建议",
            SkillPriority::Critical,
            vec![
                field("our_product", "string", "我方商品名称", true),
                field("our_price", "number", "我方价格", false),
                field(
                    "competitors",
                    "array<object>",
                    "竞品列表：{name,price,monthly_sales,rating}",
                    false,
                ),
            ],
            &["deterministic"],
        ),
        spec(
            "data_multi_period_trend",
            "多周期趋势分析",
            "多周期环比、移动平均和趋势动能分析",
            SkillPriority::Critical,
            vec![
                field(
                    "metric",
                    "string",
                    "指标名：gmv/orders/uv/conversion_rate/ad_spend，默认gmv",
                    false,
                ),
                field(
                    "periods",
                    "array<integer>",
                    "对比周期天数列表，默认[7,14,30]",
                    false,
                ),
                field("ma_window", "integer", "移动平均窗口天数，默认7", false),
                field("values", "array<number>", "手动提供的时序数据", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_channel_roi",
            "渠道ROI归因",
            "分析各渠道广告ROI、流量效率和GMV贡献",
            SkillPriority::High,
            vec![
                field("days", "integer", "分析天数，默认30", false),
                field(
                    "include_organic",
                    "boolean",
                    "是否包含自然流量估算，默认true",
                    false,
                ),
                field(
                    "channels",
                    "array<object>",
                    "渠道列表：{name,gmv,ad_spend,orders,uv}",
                    false,
                ),
            ],
            &["deterministic"],
        ),
        spec(
            "data_ltv_calculator",
            "LTV客户价值计算",
            "计算客户生命周期价值、CAC健康度和回收周期",
            SkillPriority::Critical,
            vec![
                field("days", "integer", "分析天数，默认90", false),
                field(
                    "churn_period_days",
                    "integer",
                    "流失判定天数，默认90",
                    false,
                ),
                field("cac", "number", "获客成本CAC", false),
                field("avg_order_value", "number", "客单价，默认120", false),
                field("purchase_frequency", "number", "月购买频次，默认1.6", false),
                field("gross_margin", "number", "毛利率%，默认35", false),
                field("monthly_retention", "number", "月留存率%，默认35", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_cohort_analysis",
            "队列留存分析",
            "生成队列留存曲线并识别关键流失节点",
            SkillPriority::Critical,
            vec![
                field("days", "integer", "分析总天数，默认90", false),
                field(
                    "cohort_unit",
                    "string",
                    "队列单位：week/month，默认week",
                    false,
                ),
                field("new_customers", "integer", "首期新客数，默认120", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_ab_test_analyzer",
            "A/B测试分析",
            "计算A/B测试转化率差异、卡方值、置信度和推全建议",
            SkillPriority::Critical,
            vec![
                field("control_visitors", "integer", "对照组访客数", true),
                field("control_conversions", "integer", "对照组转化数", true),
                field("test_visitors", "integer", "实验组访客数", true),
                field("test_conversions", "integer", "实验组转化数", true),
                field("test_name", "string", "测试名称", false),
                field("metric_name", "string", "测试指标名称", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_attribution_analysis",
            "渠道归因分析",
            "多触点渠道归因模型，比较末次、线性、时间衰减和位置归因",
            SkillPriority::Critical,
            vec![
                field("days", "integer", "分析天数，默认30", false),
                field(
                    "attribution_model",
                    "string",
                    "last_touch/linear/time_decay/position/all，默认all",
                    false,
                ),
                field(
                    "channels",
                    "array<object>",
                    "渠道列表：{name,gmv,uv,conversion_rate,ad_spend}",
                    false,
                ),
            ],
            &["deterministic"],
        ),
        spec(
            "data_refund_decomposition",
            "退款帕累托分析",
            "按渠道/类目/原因分解退款来源并识别关键问题项",
            SkillPriority::High,
            vec![
                field("days", "integer", "分析天数，默认30", false),
                field("top_n", "integer", "展示前N个问题项，默认5", false),
                field(
                    "items",
                    "array<object>",
                    "退款项：{name,refund_amount,refund_rate,gmv}",
                    false,
                ),
            ],
            &["deterministic"],
        ),
        spec(
            "data_seasonal_decompose",
            "季节性趋势分解",
            "STL-lite趋势、季节和残差分解",
            SkillPriority::Critical,
            vec![
                field(
                    "metric",
                    "string",
                    "分解指标：gmv/orders/uv，默认gmv",
                    false,
                ),
                field("days", "integer", "分析天数，默认56", false),
                field("period", "integer", "季节周期长度，默认7", false),
                field("values", "array<number>", "手动提供的时序数据", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_price_elasticity",
            "价格弹性分析",
            "用弧弹性公式计算需求弹性和最优价格参考",
            SkillPriority::Critical,
            vec![
                field(
                    "price_points",
                    "array<object>",
                    "历史价格-销量对：{price,quantity}",
                    false,
                ),
                field("cost", "number", "单品成本", false),
                field("current_price", "number", "当前售价", false),
                field("days", "integer", "分析天数，默认60", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_demand_forecast",
            "需求预测",
            "简单指数平滑预测未来需求并输出备货建议",
            SkillPriority::Critical,
            vec![
                field(
                    "metric",
                    "string",
                    "预测指标：gmv/orders/uv，默认orders",
                    false,
                ),
                field("days", "integer", "历史数据天数，默认60", false),
                field(
                    "forecast_days",
                    "array<integer>",
                    "预测未来N天，默认[7,14,30]",
                    false,
                ),
                field("values", "array<number>", "手动提供的历史序列", false),
            ],
            &["deterministic"],
        ),
        spec(
            "data_comprehensive_diagnosis",
            "综合业务诊断",
            "编排经营看板、漏斗、异常、退款、预测、归因和LTV，输出综合业务诊断",
            SkillPriority::Critical,
            vec![
                field("days", "integer", "分析周期，默认30", false),
                field(
                    "focus",
                    "string",
                    "重点方向：growth/profit/retention/risk/all，默认all",
                    false,
                ),
                field(
                    "values",
                    "array<number>",
                    "时序数据，用于异常、趋势和预测",
                    false,
                ),
                field(
                    "channels",
                    "array<object>",
                    "渠道列表：{name,gmv,uv,conversion_rate,ad_spend}",
                    false,
                ),
                field(
                    "items",
                    "array<object>",
                    "退款问题项：{name,refund_amount,refund_rate,gmv}",
                    false,
                ),
            ],
            &["deterministic", "orchestrator"],
        ),
    ]
}

pub struct DataSkill {
    spec: SkillSpec,
}

impl DataSkill {
    pub fn new(skill_id: &str) -> Result<Self, SkillError> {
        let spec = specs()
            .into_iter()
            .find(|spec| spec.id == skill_id)
            .ok_or_else(|| SkillError::invalid_parameter("skill_id", "known data skill id"))?;
        Ok(Self { spec })
    }
}

#[async_trait]
impl Skill for DataSkill {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        execute(&self.spec.id, params, &context).await
    }
}

pub async fn execute(skill_name: &str, input: Value, _context: &SkillContext) -> SkillResult {
    ensure_object(&input)?;
    let mut output = match skill_name {
        "query_store_metrics" => query_store_metrics(&input),
        "query_product_metrics" => query_product_metrics(&input),
        "query_campaign_metrics" => query_campaign_metrics(&input),
        "data_funnel_analysis" => funnel_analysis(&input),
        "data_anomaly_diagnosis" => anomaly_diagnosis(&input),
        "data_trend_forecast" => trend_forecast(&input),
        "data_dashboard" => data_dashboard(&input),
        "data_customer_segmentation" => customer_segmentation(&input),
        "data_competitor_analysis" => competitor_analysis(&input)?,
        "data_multi_period_trend" => multi_period_trend(&input),
        "data_channel_roi" => channel_roi(&input),
        "data_ltv_calculator" => ltv_calculator(&input),
        "data_cohort_analysis" => cohort_analysis(&input),
        "data_ab_test_analyzer" => ab_test_analyzer(&input)?,
        "data_attribution_analysis" => attribution_analysis(&input),
        "data_refund_decomposition" => refund_decomposition(&input),
        "data_seasonal_decompose" => seasonal_decompose(&input),
        "data_price_elasticity" => price_elasticity(&input),
        "data_demand_forecast" => demand_forecast(&input),
        "data_comprehensive_diagnosis" => comprehensive_diagnosis(&input),
        other => return Err(SkillError::invalid_parameter(other, "known data skill")),
    };
    normalize_data_output(skill_name, &input, &mut output);
    Ok(SkillOutcome::new(output).with_summary("data skill executed"))
}

fn spec(
    id: &str,
    name: &str,
    description: &str,
    priority: SkillPriority,
    inputs: Vec<SkillInputField>,
    tags: &[&str],
) -> SkillSpec {
    let mut skill = SkillSpec::new(id, name, description)
        .with_category(category())
        .with_priority(priority);
    for input in inputs {
        skill = skill.with_input(input);
    }
    for tag in tags {
        skill = skill.with_tag(*tag);
    }
    skill
}

fn field(name: &str, kind: &str, description: &str, required: bool) -> SkillInputField {
    let schema = match kind {
        "string" => json!({"type": "string"}),
        "integer" => json!({"type": "integer"}),
        "number" => json!({"type": "number"}),
        "boolean" => json!({"type": "boolean"}),
        "array<number>" => json!({"type": "array", "items": {"type": "number"}}),
        "array<integer>" => json!({"type": "array", "items": {"type": "integer"}}),
        "array<object>" => json!({"type": "array", "items": {"type": "object"}}),
        _ => json!({"type": "object"}),
    };
    if required {
        SkillInputField::required(name, description, schema)
    } else {
        SkillInputField::optional(name, description, schema)
    }
}

fn ensure_object(input: &Value) -> Result<(), SkillError> {
    if input.is_object() {
        Ok(())
    } else {
        Err(SkillError::invalid_parameter("$", "object"))
    }
}

fn deferred(skill: &str, reason: &str, input: &Value) -> Value {
    json!({
        "status": "deferred",
        "skill": skill,
        "deferred_reason": reason,
        "received_params": input,
        "side_effects": "none",
        "expected_outputs": ["结构化指标", "诊断摘要", "可执行建议"]
    })
}

fn query_store_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform_filter = {
        let value = s(input, "platform", "all");
        if value.trim().is_empty() {
            "all".to_owned()
        } else {
            value
        }
    };
    let include_anomalies = input
        .get("include_anomalies")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let metric_keys = [
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "previous_orders",
        "last_orders",
        "visitors",
        "uv",
        "previous_visitors",
        "last_visitors",
        "clicks",
        "click_count",
        "ad_spend",
        "ad_cost",
        "refund_amount",
        "refunds",
        "gross_profit",
        "profit",
    ];
    let platform_rows = store_metric_platform_rows(input, &platform_filter);
    let has_explicit_metrics = has_any_input(input, &metric_keys) || !platform_rows.is_empty();

    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_store_metrics",
            "has_data": false,
            "adapter_status": "store_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有真实店铺指标输入，也没有 store_metrics adapter；不会伪造平台实时数据",
            "side_effects": "none",
            "received_params": input,
            "expected_inputs": [
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "ad_spend/ad_cost",
                "refund_amount/refunds",
                "gross_profit/profit",
                "platforms[]"
            ],
            "next_actions": [
                "在本轮输入中提供店铺经营指标",
                "后续通过 service 层接入已鉴权的 store_metrics adapter",
                "前端展示为缺数据状态，不要当作平台实时读取结果"
            ],
            "summary": "店铺指标查询等待真实数据输入"
        });
    }

    let totals = store_metric_totals(input, &platform_rows);
    let previous = store_metric_previous(input);
    let gmv = value_pointer_number(&totals, "/gmv").unwrap_or_default();
    let orders = value_pointer_number(&totals, "/orders").unwrap_or_default();
    let visitors = value_pointer_number(&totals, "/uv").unwrap_or_default();
    let clicks = value_pointer_number(&totals, "/clicks").unwrap_or_default();
    let ad_spend = value_pointer_number(&totals, "/ad_spend").unwrap_or_default();
    let refund_amount = value_pointer_number(&totals, "/refund_amount").unwrap_or_default();
    let gross_profit = value_pointer_number(&totals, "/gross_profit").unwrap_or_default();

    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = if orders > 0.0 { gmv / orders } else { 0.0 };
    let ad_roi = if ad_spend > 0.0 {
        Some(round2(gmv / ad_spend))
    } else {
        None
    };
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let week_over_week = store_metric_week_over_week(gmv, orders, visitors, &previous);
    let anomalies = if include_anomalies {
        store_metric_anomalies(gmv, conversion_rate, ad_roi, refund_rate, gross_margin)
    } else {
        Vec::new()
    };
    let anomaly_count = anomalies.len();

    json!({
        "status": "success",
        "skill": "query_store_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_metrics",
        "数据来源": "用户提供指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "platform": platform_filter,
        "platforms": platform_rows,
        "totals": totals,
        "derived_metrics": {
            "conversion_rate": percent(conversion_rate, 2),
            "ctr": percent(ctr, 2),
            "aov": round2(aov),
            "ad_roi": ad_roi,
            "refund_rate": percent(refund_rate, 2),
            "gross_margin": percent(gross_margin, 2)
        },
        "week_over_week": week_over_week,
        "anomalies": anomalies,
        "anomaly_count": anomaly_count,
        "summary": format!("店铺指标已从本轮显式输入汇总，GMV={}，订单={}", round2(gmv), round2(orders)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的指标，不代表已读取淘宝、抖音或其它平台实时数据。"
    })
}

fn query_product_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = normalize_metric_platform(input);
    let product_ref = s(input, "product_ref", "");
    let product_id = s(input, "product_id", "");
    let product_name = s(input, "product_name", "");
    let sku_id = s(input, "sku_id", "");
    let metric_keys = [
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "previous_orders",
        "last_orders",
        "visitors",
        "uv",
        "clicks",
        "click_count",
        "ad_spend",
        "ad_cost",
        "refund_amount",
        "refunds",
        "gross_profit",
        "profit",
        "inventory",
        "stock",
        "stockout_count",
    ];
    let has_explicit_metrics = has_any_input(input, &metric_keys);
    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_product_metrics",
            "has_data": false,
            "adapter_status": "product_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有本轮显式商品指标输入，也没有 product_metrics adapter；不会默认读取任何预设商品或伪造平台实时商品数据",
            "side_effects": "none",
            "query": {
                "product_ref": empty_string_as_null(product_ref),
                "product_id": empty_string_as_null(product_id),
                "product_name": empty_string_as_null(product_name),
                "sku_id": empty_string_as_null(sku_id),
                "platform": platform,
                "days": days
            },
            "expected_inputs": [
                "product_ref/product_id/product_name/sku_id",
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "ad_spend/ad_cost",
                "refund_amount/refunds",
                "gross_profit/profit",
                "inventory/stock",
                "stockout_count"
            ],
            "next_actions": [
                "在本轮输入中提供商品或SKU经营指标",
                "或先通过前端引用商品并由后续 service adapter 注入真实商品指标",
                "前端展示为缺商品指标状态，不要当作已读取平台实时商品数据"
            ],
            "answerPolicy": {
                "no_default_product": true,
                "live_platform_lookup": false,
                "source": "explicit_product_metrics_required"
            },
            "summary": "商品指标查询等待真实数据输入"
        });
    }

    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (visitors, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (ad_spend, _) = number_from_any(input, &["ad_spend", "ad_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let (gross_profit, _) = number_from_any(input, &["gross_profit", "profit"], 0.0);
    let (inventory, _) = number_from_any(input, &["inventory", "stock"], 0.0);
    let (stockout_count, _) = number_from_any(input, &["stockout_count"], 0.0);
    let previous = store_metric_previous(input);
    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = if orders > 0.0 { gmv / orders } else { 0.0 };
    let ad_roi = if ad_spend > 0.0 {
        Some(round2(gmv / ad_spend))
    } else {
        None
    };
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let sell_through_pressure = if inventory > 0.0 {
        round2(orders / inventory)
    } else {
        0.0
    };
    let risks = product_metric_risks(
        gmv,
        conversion_rate,
        ad_roi,
        refund_rate,
        inventory,
        stockout_count,
    );

    json!({
        "status": "success",
        "skill": "query_product_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_product_metrics",
        "数据来源": "用户提供商品指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "query": {
            "product_ref": empty_string_as_null(product_ref),
            "product_id": empty_string_as_null(product_id),
            "product_name": empty_string_as_null(product_name),
            "sku_id": empty_string_as_null(sku_id),
            "platform": platform
        },
        "totals": {
            "gmv": round2(gmv),
            "orders": round2(orders),
            "uv": round2(visitors),
            "clicks": round2(clicks),
            "ad_spend": round2(ad_spend),
            "refund_amount": round2(refund_amount),
            "gross_profit": round2(gross_profit),
            "inventory": round2(inventory),
            "stockout_count": round2(stockout_count)
        },
        "derived_metrics": {
            "conversion_rate": percent(conversion_rate, 2),
            "ctr": percent(ctr, 2),
            "aov": round2(aov),
            "ad_roi": ad_roi,
            "refund_rate": percent(refund_rate, 2),
            "gross_margin": percent(gross_margin, 2),
            "sell_through_pressure": sell_through_pressure
        },
        "week_over_week": store_metric_week_over_week(gmv, orders, visitors, &previous),
        "risks": risks,
        "next_actions": product_metric_actions(conversion_rate, ad_roi, refund_rate, inventory, stockout_count),
        "answerPolicy": {
            "no_default_product": true,
            "live_platform_lookup": false,
            "source": "user_explicit_product_metrics"
        },
        "summary": format!("商品指标已从本轮显式输入汇总，GMV={}，订单={}", round2(gmv), round2(orders)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的商品指标，不代表已读取淘宝、抖音、千川或其它平台实时商品数据。"
    })
}

fn query_campaign_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = normalize_metric_platform(input);
    let campaign_id = s(input, "campaign_id", "");
    let campaign_name = s(input, "campaign_name", "");
    let campaign_type = s(input, "campaign_type", "general");
    let metric_keys = [
        "budget",
        "spend",
        "ad_spend",
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "visitors",
        "uv",
        "clicks",
        "click_count",
        "impressions",
        "exposure",
        "coupon_cost",
        "refund_amount",
        "refunds",
    ];
    let has_explicit_metrics = has_any_input(input, &metric_keys);
    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_campaign_metrics",
            "has_data": false,
            "adapter_status": "campaign_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有本轮显式活动指标输入，也没有 campaign_metrics adapter；不会伪造活动、投放或平台实时数据",
            "side_effects": "none",
            "query": {
                "campaign_id": empty_string_as_null(campaign_id),
                "campaign_name": empty_string_as_null(campaign_name),
                "campaign_type": campaign_type,
                "platform": platform,
                "days": days
            },
            "expected_inputs": [
                "campaign_id/campaign_name/campaign_type",
                "budget",
                "spend/ad_spend",
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "impressions/exposure",
                "coupon_cost",
                "refund_amount/refunds"
            ],
            "next_actions": [
                "在本轮输入中提供活动预算、花费、GMV、曝光、点击或订单等指标",
                "后续通过 service 层接入已鉴权的 campaign_metrics adapter",
                "前端展示为缺活动指标状态，不要当作平台实时活动复盘结果"
            ],
            "answerPolicy": {
                "live_platform_lookup": false,
                "source": "explicit_campaign_metrics_required"
            },
            "summary": "活动指标查询等待真实数据输入"
        });
    }

    let (budget, _) = number_from_any(input, &["budget"], 0.0);
    let (spend, has_spend) = number_from_any(input, &["spend", "ad_spend"], 0.0);
    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (visitors, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (impressions, _) = number_from_any(input, &["impressions", "exposure"], 0.0);
    let (coupon_cost, _) = number_from_any(input, &["coupon_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let total_cost = spend + coupon_cost;
    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, impressions.max(1.0));
    let cpc = if clicks > 0.0 {
        Some(round2(spend / clicks))
    } else {
        None
    };
    let cpm = if impressions > 0.0 {
        Some(round2(spend / impressions * 1000.0))
    } else {
        None
    };
    let roas = if spend > 0.0 {
        Some(round2(gmv / spend))
    } else {
        None
    };
    let roi = if total_cost > 0.0 {
        Some(round2((gmv - total_cost) / total_cost))
    } else {
        None
    };
    let budget_usage = pct(spend, budget.max(1.0));
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let previous = store_metric_previous(input);
    let risks = campaign_metric_risks(
        budget,
        spend,
        gmv,
        conversion_rate,
        roas,
        refund_rate,
        has_spend,
    );

    json!({
        "status": "success",
        "skill": "query_campaign_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_campaign_metrics",
        "数据来源": "用户提供活动指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "query": {
            "campaign_id": empty_string_as_null(campaign_id),
            "campaign_name": empty_string_as_null(campaign_name),
            "campaign_type": campaign_type,
            "platform": platform
        },
        "totals": {
            "budget": round2(budget),
            "spend": round2(spend),
            "coupon_cost": round2(coupon_cost),
            "total_cost": round2(total_cost),
            "gmv": round2(gmv),
            "orders": round2(orders),
            "uv": round2(visitors),
            "clicks": round2(clicks),
            "impressions": round2(impressions),
            "refund_amount": round2(refund_amount)
        },
        "derived_metrics": {
            "budget_usage": percent(budget_usage, 2),
            "ctr": percent(ctr, 2),
            "conversion_rate": percent(conversion_rate, 2),
            "cpc": cpc,
            "cpm": cpm,
            "roas": roas,
            "roi": roi,
            "refund_rate": percent(refund_rate, 2)
        },
        "week_over_week": store_metric_week_over_week(gmv, orders, visitors, &previous),
        "risks": risks,
        "next_actions": campaign_metric_actions(budget_usage, conversion_rate, roas, refund_rate),
        "answerPolicy": {
            "live_platform_lookup": false,
            "source": "user_explicit_campaign_metrics"
        },
        "summary": format!("活动指标已从本轮显式输入汇总，GMV={}，花费={}", round2(gmv), round2(spend)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的活动指标，不代表已读取淘宝、抖音、千川或其它平台实时活动数据。"
    })
}

fn normalize_data_output(skill_name: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }

    if data_output_needs_user_input(map) {
        let reason = data_deferred_reason(map);
        map.insert("status".to_owned(), json!("deferred"));
        map.insert("skill".to_owned(), json!(skill_name));
        map.insert("has_data".to_owned(), json!(false));
        map.insert("adapter_status".to_owned(), json!("user_input_required"));
        map.insert("data_readiness".to_owned(), json!("missing_user_input"));
        map.insert("deferred_reason".to_owned(), json!(reason));
        map.insert("summary".to_owned(), json!(data_summary(skill_name)));
        return;
    }

    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(data_readiness(skill_name, input)),
    );
    map.insert("summary".to_owned(), json!(data_summary(skill_name)));
}

fn data_output_needs_user_input(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("error") || matches!(map.get("has_data").and_then(Value::as_bool), Some(false))
}

fn data_deferred_reason(map: &serde_json::Map<String, Value>) -> String {
    ["deferred_reason", "error", "hint", "提示"]
        .iter()
        .find_map(|key| {
            map.get(*key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "需要补充用户输入数据".to_owned())
}

fn data_readiness(skill_name: &str, input: &Value) -> &'static str {
    match skill_name {
        "data_customer_segmentation"
            if !has_any_input(
                input,
                &[
                    "total_customers",
                    "avg_order_value",
                    "avg_orders_per_customer",
                    "recency_days",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "data_competitor_analysis" if input_array_missing_or_empty(input, "competitors") => {
            "calculated_from_defaults"
        }
        "data_ltv_calculator"
            if !has_any_input(
                input,
                &[
                    "cac",
                    "avg_order_value",
                    "purchase_frequency",
                    "gross_margin",
                    "monthly_retention",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "data_cohort_analysis" if !has_any_input(input, &["new_customers"]) => {
            "calculated_from_defaults"
        }
        "data_price_elasticity" if input_array_missing_or_empty(input, "price_points") => {
            "calculated_from_defaults"
        }
        _ => "user_provided",
    }
}

fn data_summary(skill_name: &str) -> &'static str {
    match skill_name {
        "query_store_metrics" => "店铺指标查询结果已生成",
        "query_product_metrics" => "商品指标查询结果已生成",
        "query_campaign_metrics" => "活动指标查询结果已生成",
        "data_anomaly_diagnosis" => "异常诊断报告已生成",
        "data_trend_forecast" => "趋势预测报告已生成",
        "data_customer_segmentation" => "客户分层报告已生成",
        "data_competitor_analysis" => "竞品分析报告已生成",
        "data_multi_period_trend" => "多周期趋势分析报告已生成",
        "data_attribution_analysis" => "渠道归因分析报告已生成",
        "data_ltv_calculator" => "LTV客户价值报告已生成",
        "data_cohort_analysis" => "队列留存分析报告已生成",
        "data_ab_test_analyzer" => "A/B测试分析报告已生成",
        "data_seasonal_decompose" => "季节性趋势分解报告已生成",
        "data_price_elasticity" => "价格弹性分析报告已生成",
        "data_demand_forecast" => "需求预测报告已生成",
        "data_comprehensive_diagnosis" => "综合业务诊断报告已生成",
        _ => "数据分析报告已生成",
    }
}

fn funnel_analysis(input: &Value) -> Value {
    let impressions = i(input, "impressions", 0).max(i(input, "clicks", 0));
    let clicks = i(input, "clicks", 0);
    let orders = i(input, "orders", 0);
    let add_to_cart = i(input, "add_to_cart", (orders as f64 * 2.5).round() as i64);
    let payments = i(input, "payments", (orders as f64 * 0.95).round() as i64);
    if impressions <= 0 && clicks <= 0 {
        return deferred(
            "data_funnel_analysis",
            "需要曝光、点击、加购、下单或付款等漏斗数据",
            input,
        );
    }

    let stages = [
        (
            "曝光→点击",
            impressions,
            clicks,
            5.0,
            "优化主图和标题关键词，提升点击率",
        ),
        (
            "点击→加购",
            clicks,
            add_to_cart,
            30.0,
            "改善详情页、增强价格竞争力",
        ),
        (
            "加购→下单",
            add_to_cart,
            orders,
            35.0,
            "设置限时优惠和库存紧迫感",
        ),
        (
            "下单→付款",
            orders,
            payments,
            90.0,
            "简化支付流程，增加支付方式",
        ),
    ];
    let mut bottleneck = "无明显瓶颈";
    let mut worst_gap = f64::MIN;
    let funnel = stages
        .iter()
        .filter(|(_, upper, _, _, _)| *upper > 0)
        .map(|(name, upper, lower, benchmark, _)| {
            let rate = pct(*lower as f64, *upper as f64);
            let gap = benchmark - rate;
            if gap > worst_gap {
                worst_gap = gap;
                bottleneck = name;
            }
            json!({
                "阶段": name,
                "转化率": percent(rate, 2),
                "基准值": percent(*benchmark, 1),
                "差距": percent(gap, 2),
                "状态": if gap <= 0.0 { "正常" } else if gap < 5.0 { "需关注" } else { "异常" }
            })
        })
        .collect::<Vec<_>>();
    let suggestion = stages
        .iter()
        .find(|(name, _, _, _, _)| *name == bottleneck)
        .map(|(_, _, _, _, suggestion)| *suggestion)
        .unwrap_or("持续监控各环节");

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": format!("漏斗分析完成，瓶颈环节：{bottleneck}"),
        "数据来源": "用户提供",
        "漏斗分析": funnel,
        "整体转化率": percent(pct(payments as f64, impressions.max(1) as f64), 3),
        "瓶颈环节": bottleneck,
        "优化建议": suggestion
    })
}

fn anomaly_diagnosis(input: &Value) -> Value {
    let values = nums(input, "values");
    if values.len() < 3 {
        return json!({"error": "无真实数据且未提供足够时序数据", "required": "values至少3个数字"});
    }
    let threshold = n(input, "threshold", 2.0).abs().max(0.1);
    let mean = average(&values);
    let std = stddev(&values).max(1e-9);
    let anomalies = values
        .iter()
        .enumerate()
        .filter_map(|(idx, value)| {
            let z = (*value - mean) / std;
            (z.abs() >= threshold).then(|| {
                json!({"第几天": idx + 1, "数值": round2(*value), "Z-Score": round2(z), "方向": if z > 0.0 { "激增" } else { "骤降" }})
            })
        })
        .collect::<Vec<_>>();
    json!({
        "数据来源": "用户提供",
        "指标": s(input, "metric_name", "未知指标"),
        "样本数": values.len(),
        "均值": round2(mean),
        "标准差": round2(std),
        "异常点数": anomalies.len(),
        "异常详情": anomalies,
        "建议": if anomalies.is_empty() { "指标运行正常" } else { "立即排查异常原因" }
    })
}

fn trend_forecast(input: &Value) -> Value {
    let values = nums(input, "values");
    if values.len() < 2 {
        return json!({"error": "数据不足，无法预测", "hint": "请提供 values 至少2个数据点"});
    }
    let periods = i(input, "forecast_periods", 7).max(1) as usize;
    let (slope, intercept) = linear_fit(&values);
    let forecasts = (0..periods)
        .map(|idx| json!({"期数": format!("第{}天", idx + 1), "预测值": round2((intercept + slope * (values.len() + idx) as f64).max(0.0))}))
        .collect::<Vec<_>>();
    let avg_growth = avg_growth(&values);
    let trend = if slope > average(&values).abs() * 0.01 {
        "上升"
    } else if slope < -average(&values).abs() * 0.01 {
        "下降"
    } else {
        "平稳"
    };
    json!({
        "数据来源": "用户提供",
        "指标": metric_display(&s(input, "metric_name", "gmv")),
        "历史数据点数": values.len(),
        "最新值": round2(*values.last().unwrap_or(&0.0)),
        "趋势方向": trend,
        "平均日增长率": percent(avg_growth, 2),
        "斜率": round4(slope),
        "预测结果": forecasts,
        "置信说明": "基于线性回归，反映近期趋势，不含季节性因素"
    })
}

fn data_dashboard(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = s(input, "platform", "all");
    let (gmv, has_gmv) = number_from_any(input, &["gmv", "revenue"], 128_000.0);
    let (previous_gmv, has_previous_gmv) =
        number_from_any(input, &["previous_gmv", "last_gmv"], gmv * 0.92);
    let (orders, has_orders) = number_from_any(input, &["orders", "order_count"], 1_280.0);
    let (previous_orders, has_previous_orders) =
        number_from_any(input, &["previous_orders", "last_orders"], orders * 0.95);
    let (visitors, has_visitors) = number_from_any(input, &["visitors", "uv"], 32_000.0);
    let (previous_visitors, has_previous_visitors) = number_from_any(
        input,
        &["previous_visitors", "last_visitors"],
        visitors * 0.96,
    );
    let (clicks, has_clicks) = number_from_any(input, &["clicks", "click_count"], visitors * 0.18);
    let (ad_spend, has_ad_spend) = number_from_any(input, &["ad_spend", "ad_cost"], gmv * 0.12);
    let (refund_amount, has_refund_amount) =
        number_from_any(input, &["refund_amount", "refunds"], gmv * 0.035);
    let (gross_profit, has_gross_profit) =
        number_from_any(input, &["gross_profit", "profit"], gmv * 0.28);
    let (target_gmv, has_target_gmv) =
        number_from_any(input, &["target_gmv", "goal_gmv"], gmv * 1.15);

    let has_user_metrics = [
        has_gmv,
        has_previous_gmv,
        has_orders,
        has_previous_orders,
        has_visitors,
        has_previous_visitors,
        has_clicks,
        has_ad_spend,
        has_refund_amount,
        has_gross_profit,
        has_target_gmv,
    ]
    .into_iter()
    .any(|item| item);

    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = gmv / orders.max(1.0);
    let ad_roi = gmv / ad_spend.max(0.01);
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let target_achievement = pct(gmv, target_gmv.max(1.0));
    let gmv_growth = pct_delta(gmv, previous_gmv);
    let order_growth = pct_delta(orders, previous_orders);
    let visitor_growth = pct_delta(visitors, previous_visitors);

    let mut alerts = Vec::new();
    if gmv_growth < -10.0 {
        alerts.push(json!({"level": "warning", "metric": "GMV", "message": "GMV环比下滑超过10%，优先排查流量、转化和客单价"}));
    }
    if conversion_rate < 2.0 {
        alerts.push(json!({"level": "warning", "metric": "转化率", "message": "访客到订单转化偏低，建议复核详情页、价格和首屏卖点"}));
    }
    if ad_roi < 2.0 {
        alerts.push(json!({"level": "warning", "metric": "广告ROI", "message": "广告ROI低于2，建议收缩低效计划并复盘素材"}));
    }
    if refund_rate > 6.0 {
        alerts.push(json!({"level": "warning", "metric": "退款率", "message": "退款率偏高，建议拆解售后原因和商品描述一致性"}));
    }
    if gross_margin < 20.0 {
        alerts.push(json!({"level": "warning", "metric": "毛利率", "message": "毛利率偏低，需复核进货成本、优惠力度和平台扣点"}));
    }
    if alerts.is_empty() {
        alerts.push(json!({"level": "info", "metric": "整体", "message": "核心指标未触发高优先级预警，建议继续跟踪趋势和渠道结构"}));
    }

    let score = dashboard_health_score(
        gmv_growth,
        conversion_rate,
        ad_roi,
        refund_rate,
        gross_margin,
        target_achievement,
    );
    let data_readiness = if has_user_metrics {
        "user_provided"
    } else {
        "sample_only"
    };

    json!({
        "status": "success",
        "has_data": has_user_metrics,
        "data_readiness": data_readiness,
        "adapter_status": if has_user_metrics { "manual_input" } else { "store_metrics_required" },
        "数据来源": if has_user_metrics { "用户提供指标" } else { "未接入 store_metrics adapter；以下为样例基线，仅用于展示看板结构和诊断口径" },
        "周期": format!("{days}天"),
        "平台": platform,
        "核心指标": [
            metric_card("GMV", round2(gmv), "CNY", gmv_growth, "看收入规模和环比动能"),
            metric_card("订单数", round2(orders), "orders", order_growth, "看成交规模"),
            metric_card("访客数", round2(visitors), "UV", visitor_growth, "看流量盘子"),
            metric_card("点击率", round2(ctr), "%", ctr - 15.0, "流量点击效率"),
            metric_card("转化率", round2(conversion_rate), "%", conversion_rate - 3.0, "访客到订单效率"),
            metric_card("客单价", round2(aov), "CNY", 0.0, "成交结构和价格带"),
            metric_card("广告ROI", round2(ad_roi), "x", ad_roi - 2.5, "投放产出效率"),
            metric_card("退款率", round2(refund_rate), "%", 4.0 - refund_rate, "售后和描述一致性"),
            metric_card("毛利率", round2(gross_margin), "%", gross_margin - 25.0, "利润安全垫"),
            metric_card("目标完成率", round2(target_achievement), "%", target_achievement - 100.0, "目标进度")
        ],
        "经营健康度": {
            "总分": score,
            "等级": if score >= 80 { "健康" } else if score >= 60 { "需关注" } else { "风险" },
            "评分口径": "GMV动能、转化、ROI、退款、毛利和目标完成率的确定性规则评分"
        },
        "看板分区": [
            {"section": "增长", "metrics": ["GMV", "订单数", "访客数"], "summary": growth_summary(gmv_growth, order_growth, visitor_growth)},
            {"section": "效率", "metrics": ["转化率", "客单价", "广告ROI"], "summary": efficiency_summary(conversion_rate, ad_roi, aov)},
            {"section": "风险", "metrics": ["退款率", "毛利率"], "summary": risk_summary(refund_rate, gross_margin)}
        ],
        "预警": alerts,
        "下一步动作": dashboard_actions(gmv_growth, conversion_rate, ad_roi, refund_rate, gross_margin),
        "说明": if has_user_metrics { "本看板基于本次 skill 入参计算，不代表外部平台实时拉取结果。" } else { "当前没有真实店铺指标入参，也没有 store_metrics adapter；前端应展示为缺数据/样例态，不能当作平台实时数据。" }
    })
}

fn store_metric_platform_rows(input: &Value, platform_filter: &str) -> Vec<Value> {
    input
        .get("platforms")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    platform_filter == "all"
                        || platform_filter.trim().is_empty()
                        || value_string(item, "platform", "")
                            .eq_ignore_ascii_case(platform_filter.trim())
                })
                .filter_map(|item| {
                    let platform = value_string(item, "platform", "unknown");
                    let (gmv, has_gmv) = number_from_any(item, &["gmv", "revenue"], 0.0);
                    let (orders, has_orders) =
                        number_from_any(item, &["orders", "order_count"], 0.0);
                    let (uv, has_uv) = number_from_any(item, &["visitors", "uv"], 0.0);
                    let (clicks, has_clicks) =
                        number_from_any(item, &["clicks", "click_count"], 0.0);
                    let (ad_spend, has_ad_spend) =
                        number_from_any(item, &["ad_spend", "ad_cost"], 0.0);
                    let (refund_amount, has_refund) =
                        number_from_any(item, &["refund_amount", "refunds"], 0.0);
                    let (gross_profit, has_profit) =
                        number_from_any(item, &["gross_profit", "profit"], 0.0);

                    if [
                        has_gmv,
                        has_orders,
                        has_uv,
                        has_clicks,
                        has_ad_spend,
                        has_refund,
                        has_profit,
                    ]
                    .into_iter()
                    .any(|value| value)
                    {
                        Some(json!({
                            "platform": platform,
                            "gmv": round2(gmv),
                            "orders": round2(orders),
                            "uv": round2(uv),
                            "clicks": round2(clicks),
                            "ad_spend": round2(ad_spend),
                            "refund_amount": round2(refund_amount),
                            "gross_profit": round2(gross_profit),
                            "conversion_rate": percent(pct(orders, uv.max(1.0)), 2)
                        }))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn store_metric_totals(input: &Value, platform_rows: &[Value]) -> Value {
    if !platform_rows.is_empty() {
        let sum = |key: &str| {
            platform_rows
                .iter()
                .filter_map(|row| value_pointer_number(row, &format!("/{key}")))
                .sum::<f64>()
        };
        return json!({
            "gmv": round2(sum("gmv")),
            "orders": round2(sum("orders")),
            "uv": round2(sum("uv")),
            "clicks": round2(sum("clicks")),
            "ad_spend": round2(sum("ad_spend")),
            "refund_amount": round2(sum("refund_amount")),
            "gross_profit": round2(sum("gross_profit"))
        });
    }

    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (uv, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (ad_spend, _) = number_from_any(input, &["ad_spend", "ad_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let (gross_profit, _) = number_from_any(input, &["gross_profit", "profit"], 0.0);

    json!({
        "gmv": round2(gmv),
        "orders": round2(orders),
        "uv": round2(uv),
        "clicks": round2(clicks),
        "ad_spend": round2(ad_spend),
        "refund_amount": round2(refund_amount),
        "gross_profit": round2(gross_profit)
    })
}

fn store_metric_previous(input: &Value) -> Value {
    let (previous_gmv, has_previous_gmv) =
        number_from_any(input, &["previous_gmv", "last_gmv"], 0.0);
    let (previous_orders, has_previous_orders) =
        number_from_any(input, &["previous_orders", "last_orders"], 0.0);
    let (previous_visitors, has_previous_visitors) =
        number_from_any(input, &["previous_visitors", "last_visitors"], 0.0);

    json!({
        "gmv": if has_previous_gmv { json!(round2(previous_gmv)) } else { Value::Null },
        "orders": if has_previous_orders { json!(round2(previous_orders)) } else { Value::Null },
        "uv": if has_previous_visitors { json!(round2(previous_visitors)) } else { Value::Null }
    })
}

fn store_metric_week_over_week(gmv: f64, orders: f64, visitors: f64, previous: &Value) -> Value {
    let mut entries = Map::new();
    if let Some(previous_gmv) = value_pointer_number(previous, "/gmv").filter(|value| *value > 0.0)
    {
        entries.insert(
            "gmv".to_owned(),
            json!({
                "current": round2(gmv),
                "previous": round2(previous_gmv),
                "change_rate": signed_percent(pct_delta(gmv, previous_gmv), 2)
            }),
        );
    }
    if let Some(previous_orders) =
        value_pointer_number(previous, "/orders").filter(|value| *value > 0.0)
    {
        entries.insert(
            "orders".to_owned(),
            json!({
                "current": round2(orders),
                "previous": round2(previous_orders),
                "change_rate": signed_percent(pct_delta(orders, previous_orders), 2)
            }),
        );
    }
    if let Some(previous_visitors) =
        value_pointer_number(previous, "/uv").filter(|value| *value > 0.0)
    {
        entries.insert(
            "uv".to_owned(),
            json!({
                "current": round2(visitors),
                "previous": round2(previous_visitors),
                "change_rate": signed_percent(pct_delta(visitors, previous_visitors), 2)
            }),
        );
    }
    Value::Object(entries)
}

fn store_metric_anomalies(
    gmv: f64,
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    gross_margin: f64,
) -> Vec<Value> {
    let mut anomalies = Vec::new();
    if gmv <= 0.0 {
        anomalies.push(json!({
            "metric": "gmv",
            "level": "warning",
            "message": "GMV为0或未提供，后续收入分析可信度不足"
        }));
    }
    if conversion_rate > 0.0 && conversion_rate < 1.0 {
        anomalies.push(json!({
            "metric": "conversion_rate",
            "level": "warning",
            "message": "转化率低于1%，建议优先复核商品详情页、价格和客服承接"
        }));
    }
    if matches!(ad_roi, Some(value) if value > 0.0 && value < 1.5) {
        anomalies.push(json!({
            "metric": "ad_roi",
            "level": "warning",
            "message": "广告ROI低于1.5，建议检查低效计划和素材疲劳"
        }));
    }
    if refund_rate > 8.0 {
        anomalies.push(json!({
            "metric": "refund_rate",
            "level": "warning",
            "message": "退款率高于8%，建议拆解售后原因"
        }));
    }
    if gross_margin > 0.0 && gross_margin < 15.0 {
        anomalies.push(json!({
            "metric": "gross_margin",
            "level": "warning",
            "message": "毛利率低于15%，需复核成本、优惠和平台扣点"
        }));
    }
    anomalies
}

fn product_metric_risks(
    gmv: f64,
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    inventory: f64,
    stockout_count: f64,
) -> Vec<Value> {
    let mut risks = store_metric_anomalies(gmv, conversion_rate, ad_roi, refund_rate, 0.0);
    if inventory > 0.0 && inventory < 10.0 {
        risks.push(json!({
            "metric": "inventory",
            "level": "warning",
            "message": "库存低于10件，若仍有投放或活动承接，需检查补货和售罄风险"
        }));
    }
    if stockout_count > 0.0 {
        risks.push(json!({
            "metric": "stockout_count",
            "level": "warning",
            "message": "本轮输入存在缺货次数，建议评估缺货对转化和投放浪费的影响"
        }));
    }
    risks
}

fn product_metric_actions(
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    inventory: f64,
    stockout_count: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if conversion_rate > 0.0 && conversion_rate < 2.0 {
        actions.push("优先复核商品详情页卖点、价格带、评价区和客服承接，先提高商品转化率");
    }
    if matches!(ad_roi, Some(value) if value > 0.0 && value < 2.0) {
        actions.push("收缩低ROI商品投放计划，检查素材疲劳和关键词/人群匹配");
    }
    if refund_rate > 6.0 {
        actions.push("拆解退款原因，检查商品描述、规格、物流和售后口径是否一致");
    }
    if inventory > 0.0 && inventory < 10.0 {
        actions.push("低库存商品暂停大额加码，先确认补货周期和安全库存");
    }
    if stockout_count > 0.0 {
        actions.push("把缺货时段与广告消耗、访客和客服咨询量对齐，评估损失并补库存策略");
    }
    if actions.is_empty() {
        actions.push("维持当前商品节奏，下一轮重点观察转化率、ROI、退款率和库存周转");
    }
    actions
}

fn campaign_metric_risks(
    budget: f64,
    spend: f64,
    gmv: f64,
    conversion_rate: f64,
    roas: Option<f64>,
    refund_rate: f64,
    has_spend: bool,
) -> Vec<Value> {
    let mut risks = Vec::new();
    if has_spend && spend <= 0.0 {
        risks.push(json!({
            "metric": "spend",
            "level": "warning",
            "message": "活动花费为0，投放效率和ROI判断可信度不足"
        }));
    }
    if budget > 0.0 && spend > budget {
        risks.push(json!({
            "metric": "budget_usage",
            "level": "warning",
            "message": "活动花费已超过预算，建议检查预算控制和投放节奏"
        }));
    }
    if gmv <= 0.0 {
        risks.push(json!({
            "metric": "gmv",
            "level": "warning",
            "message": "活动GMV为0或未提供，无法判断活动产出"
        }));
    }
    if conversion_rate > 0.0 && conversion_rate < 1.0 {
        risks.push(json!({
            "metric": "conversion_rate",
            "level": "warning",
            "message": "活动转化率低于1%，需复核人群、价格、落地页和优惠承接"
        }));
    }
    if matches!(roas, Some(value) if value > 0.0 && value < 2.0) {
        risks.push(json!({
            "metric": "roas",
            "level": "warning",
            "message": "活动ROAS低于2，建议压缩低效计划或调整优惠机制"
        }));
    }
    if refund_rate > 8.0 {
        risks.push(json!({
            "metric": "refund_rate",
            "level": "warning",
            "message": "活动退款率偏高，可能存在冲动购买、描述不符或履约问题"
        }));
    }
    risks
}

fn campaign_metric_actions(
    budget_usage: f64,
    conversion_rate: f64,
    roas: Option<f64>,
    refund_rate: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if budget_usage > 100.0 {
        actions.push("先冻结超预算活动单元，复核预算分配和出价策略");
    }
    if conversion_rate > 0.0 && conversion_rate < 1.5 {
        actions.push("优化活动落地页、利益点表达和客服承接，优先修正转化漏斗");
    }
    if matches!(roas, Some(value) if value > 0.0 && value < 2.0) {
        actions.push("按计划/素材/人群拆ROAS，保留高产出单元，暂停低效单元");
    }
    if refund_rate > 8.0 {
        actions.push("把活动订单按SKU和退款原因拆解，避免用高退款活动继续放量");
    }
    if actions.is_empty() {
        actions.push("活动指标未见明显高危信号，下一轮继续按预算消耗、ROAS和转化率复盘");
    }
    actions
}

fn normalize_metric_platform(input: &Value) -> String {
    let platform = s(input, "platform", "all");
    if platform.trim().is_empty() {
        "all".to_owned()
    } else {
        platform
    }
}

fn empty_string_as_null(value: String) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

fn customer_segmentation(input: &Value) -> Value {
    let total = i(input, "total_customers", 1000).max(1);
    let aov = n(input, "avg_order_value", 120.0);
    let freq = n(input, "avg_orders_per_customer", 1.8);
    let recency = n(input, "recency_days", 30.0);
    let high_value = ((total as f64) * (0.12 + (freq / 20.0).min(0.08))).round() as i64;
    let loyal = ((total as f64) * 0.18).round() as i64;
    let attention = ((total as f64) * if recency <= 30.0 { 0.30 } else { 0.24 }).round() as i64;
    let lost = (total - high_value - loyal - attention).max(0);
    json!({
        "数据来源": "用户提供/默认估算",
        "客户总数": total,
        "RFM输入": {"平均客单价": round2(aov), "人均订单数": round2(freq), "平均购买间隔": round2(recency)},
        "分群": [
            {"客群标签": "Champions（冠军）", "客户数": high_value, "占比": percent(pct(high_value as f64, total as f64), 1), "策略": "专属折扣奖励，新品优先体验"},
            {"客群标签": "Loyal（忠诚客户）", "客户数": loyal, "占比": percent(pct(loyal as f64, total as f64), 1), "策略": "会员积分升级，推荐关联商品"},
            {"客群标签": "Need Attention（需关注）", "客户数": attention, "占比": percent(pct(attention as f64, total as f64), 1), "策略": "限时活动唤醒，个性化push"},
            {"客群标签": "Lost（已流失）", "客户数": lost, "占比": percent(pct(lost as f64, total as f64), 1), "策略": "低成本召回测试，无响应则停止触达"}
        ],
        "加权RFM说明": "R×0.15 + F×0.28 + M×0.57，当前为聚合输入估算"
    })
}

fn competitor_analysis(input: &Value) -> Result<Value, SkillError> {
    let product = required_string(input, "our_product")?;
    let our_price = n(input, "our_price", 100.0).max(0.01);
    let competitors = input
        .get("competitors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| {
            vec![
                json!({"name": "竞品A", "price": our_price * 0.89, "monthly_sales": 5000, "rating": 4.7}),
                json!({"name": "竞品B", "price": our_price * 1.10, "monthly_sales": 3000, "rating": 4.5}),
            ]
        });
    let prices = competitors
        .iter()
        .map(|item| n(item, "price", our_price))
        .collect::<Vec<_>>();
    let avg_price = if prices.is_empty() {
        our_price
    } else {
        average(&prices)
    };
    let matrix = competitors
        .iter()
        .map(|item| {
            let price = n(item, "price", our_price).max(0.01);
            let diff = pct(our_price - price, price);
            json!({
                "竞品": s(item, "name", "未知"),
                "价格": round2(price),
                "月销量": i(item, "monthly_sales", 0),
                "评分": n(item, "rating", 0.0),
                "价格差异": signed_percent(diff, 1),
                "我方优势": if diff < 0.0 { "价格更低" } else { "需要用卖点或服务支撑溢价" }
            })
        })
        .collect::<Vec<_>>();
    json_ok(json!({
        "我方商品": product,
        "我方价格": round2(our_price),
        "竞品矩阵": matrix,
        "价格定位": if our_price < avg_price * 0.9 { "低价位" } else if our_price < avg_price * 1.1 { "中价位" } else { "高价位" },
        "竞品均价": round2(avg_price),
        "策略建议": ["明确差异化卖点", "监控最低价竞品", "用赠品/服务降低直接比价"]
    }))
}

fn multi_period_trend(input: &Value) -> Value {
    let values = nums(input, "values");
    if values.len() < 4 {
        return json!({"error": "数据不足", "hint": "请提供 values 至少4个数据点"});
    }
    let periods = ints(input, "periods", vec![7, 14, 30]);
    let ma_window = i(input, "ma_window", 7).max(1) as usize;
    let comparisons = periods
        .into_iter()
        .filter(|period| (*period as usize) * 2 <= values.len())
        .map(|period| {
            let p = period as usize;
            let recent = average(&values[values.len() - p..]);
            let previous = average(&values[values.len() - p * 2..values.len() - p]);
            json!({"周期": format!("{period}天"), "近期均值": round2(recent), "前期均值": round2(previous), "环比": signed_percent(pct(recent - previous, previous), 2)})
        })
        .collect::<Vec<_>>();
    let ma = moving_average(&values, ma_window);
    let (slope, _) = linear_fit(&values);
    json!({
        "指标": metric_display(&s(input, "metric", "gmv")),
        "数据点数": values.len(),
        "最新值": round2(*values.last().unwrap_or(&0.0)),
        "移动平均(最近5天)": ma.into_iter().rev().take(5).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>(),
        "多周期环比": comparisons,
        "趋势动能": if slope > 0.0 { "上行" } else if slope < 0.0 { "下行" } else { "平稳" },
        "斜率": round4(slope)
    })
}

fn channel_roi(input: &Value) -> Value {
    let channels = input
        .get("channels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if channels.is_empty() {
        return deferred(
            "data_channel_roi",
            "需要真实渠道GMV、广告花费、订单和UV数据",
            input,
        );
    }
    let mut rows = channels
        .iter()
        .map(|channel| {
            let gmv = n(channel, "gmv", 0.0);
            let ad = n(channel, "ad_spend", 0.0);
            let orders = i(channel, "orders", 0).max(1) as f64;
            let uv = i(channel, "uv", 0).max(1) as f64;
            let roi = if ad > 0.0 { gmv / ad } else { 0.0 };
            json!({
                "渠道": s(channel, "name", "未知渠道"),
                "GMV贡献": round2(gmv),
                "广告花费": round2(ad),
                "广告ROI": round2(roi),
                "获客成本(CPO)": round2(ad / orders),
                "点击成本(CPC)": round4(ad / uv)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| n(b, "广告ROI", 0.0).total_cmp(&n(a, "广告ROI", 0.0)));
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": "渠道ROI分析完成",
        "渠道ROI排名": rows,
        "优化建议": ["加码高ROI渠道", "ROI低于2的渠道优先优化创意和人群"]
    })
}

fn attribution_analysis(input: &Value) -> Value {
    let channels = input
        .get("channels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if channels.is_empty() {
        return attribution_deferred(
            "data_attribution_analysis",
            "需要多渠道GMV、UV、转化率和广告花费数据",
            input,
        );
    }

    let mut rows = channels
        .iter()
        .filter_map(|channel| {
            let name = s(channel, "name", "");
            let gmv = n(channel, "gmv", 0.0).max(0.0);
            let uv = n(channel, "uv", 0.0).max(0.0);
            let conversion_rate =
                normalized_rate(n(channel, "conversion_rate", n(channel, "conv", 0.0)));
            let ad_spend = n(channel, "ad_spend", 0.0).max(0.0);
            ((gmv > 0.0 || uv > 0.0) && !name.trim().is_empty()).then_some(AttributionChannel {
                name,
                gmv,
                uv,
                conversion_rate,
                ad_spend,
            })
        })
        .collect::<Vec<_>>();

    if rows.len() < 2 {
        return attribution_deferred(
            "data_attribution_analysis",
            "归因分析至少需要2个有效渠道",
            input,
        );
    }
    rows.sort_by(|a, b| b.gmv.total_cmp(&a.gmv));

    let total_gmv = rows.iter().map(|channel| channel.gmv).sum::<f64>();
    if total_gmv <= 0.0 {
        return attribution_deferred(
            "data_attribution_analysis",
            "总GMV为0，无法计算渠道归因",
            input,
        );
    }

    let n_channels = rows.len();
    let last_touch_weights = one_hot(n_channels, 0);
    let linear_weights = normalize(vec![1.0; n_channels]);
    let decay_signals = rows
        .iter()
        .map(|channel| {
            if channel.ad_spend > 0.0 {
                channel.ad_spend
            } else {
                channel.conversion_rate * channel.uv
            }
        })
        .collect::<Vec<_>>();
    let time_decay_weights = if decay_signals.iter().all(|value| value.abs() < 1e-9) {
        linear_weights.clone()
    } else {
        normalize(decay_signals)
    };
    let position_weights = position_based_weights(n_channels);
    let requested_model = s(input, "attribution_model", "all");
    let selected_models = attribution_models_to_show(&requested_model);
    let model_map = [
        ("last_touch", "末次归因", last_touch_weights),
        ("linear", "线性归因", linear_weights),
        ("time_decay", "时间衰减归因", time_decay_weights.clone()),
        ("position", "位置归因(U形)", position_weights),
    ];

    let channel_details = rows
        .iter()
        .enumerate()
        .map(|(idx, channel)| {
            let actual_share = channel.gmv / total_gmv;
            let mut attributed = Map::new();
            for (key, display, weights) in &model_map {
                if !selected_models.iter().any(|model| model == key) {
                    continue;
                }
                let weight = weights[idx];
                let attributed_gmv = total_gmv * weight;
                let vs_actual = pct(weight - actual_share, actual_share.max(0.001));
                attributed.insert(
                    (*display).to_owned(),
                    json!({
                        "归因GMV": round2(attributed_gmv),
                        "归因权重": percent(weight * 100.0, 1),
                        "vs实际占比": signed_percent(vs_actual, 1),
                    }),
                );
            }
            json!({
                "渠道": channel.name,
                "实际GMV": round2(channel.gmv),
                "实际GMV占比": percent(actual_share * 100.0, 1),
                "UV": round2(channel.uv),
                "转化率": percent(channel.conversion_rate * 100.0, 2),
                "广告花费": round2(channel.ad_spend),
                "归因GMV对比": Value::Object(attributed),
            })
        })
        .collect::<Vec<_>>();

    let mut insights = Vec::new();
    for (idx, channel) in rows.iter().enumerate() {
        let actual_share = channel.gmv / total_gmv;
        let contribution = time_decay_weights[idx];
        let gap = contribution - actual_share;
        if gap > 0.10 {
            insights.push(format!(
                "{}：归因权重比实际GMV占比高{}，可能被低估，建议增加预算或资源",
                channel.name,
                percent(gap * 100.0, 1)
            ));
        } else if gap < -0.10 {
            insights.push(format!(
                "{}：归因权重比实际GMV占比低{}，可能被高估，需复核预算效率",
                channel.name,
                percent(gap.abs() * 100.0, 1)
            ));
        }
    }
    if insights.is_empty() {
        insights.push("各渠道贡献相对均衡，暂无明显高估/低估".to_owned());
    }

    let best_idx = time_decay_weights
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": format!("渠道归因分析完成，{}归因贡献最高", rows[best_idx].name),
        "数据来源": "用户提供渠道数据",
        "分析周期": format!("{}天", i(input, "days", 30).max(1)),
        "归因模型": if requested_model == "all" { "all" } else { requested_model.as_str() },
        "分析说明": "末次归因=成交最高渠道；线性=等权分配；时间衰减=以广告花费或转化信号代理近期触点；位置归因=首末各40%中间20%",
        "总GMV": round2(total_gmv),
        "渠道数": rows.len(),
        "各渠道归因明细": channel_details,
        "关键洞察": insights,
        "预算再分配建议": format!("时间衰减模型下，{}归因贡献最高，建议优先保障该渠道预算，同时复核低归因高花费渠道。", rows[best_idx].name),
    })
}

fn attribution_deferred(skill: &str, reason: &str, input: &Value) -> Value {
    let mut output = deferred(skill, reason, input);
    if let Some(map) = output.as_object_mut() {
        map.insert("summary".to_owned(), json!(data_summary(skill)));
    }
    output
}

fn ltv_calculator(input: &Value) -> Value {
    let aov = n(input, "avg_order_value", 120.0);
    let freq = n(input, "purchase_frequency", 1.6);
    let margin = n(input, "gross_margin", 35.0) / 100.0;
    let retention = (n(input, "monthly_retention", 35.0) / 100.0).clamp(0.01, 0.95);
    let cac = n(input, "cac", 35.0);
    let ltv = aov * freq * margin / (1.0 - retention);
    json!({
        "数据来源": "用户提供/默认估算",
        "LTV": round2(ltv),
        "月贡献毛利": round2(aov * freq * margin),
        "CAC": round2(cac),
        "LTV/CAC": round2(ltv / cac.max(0.01)),
        "回收周期(月)": round2(cac / (aov * freq * margin).max(0.01)),
        "健康度": if ltv / cac.max(0.01) >= 3.0 { "健康" } else if ltv / cac.max(0.01) >= 1.5 { "可优化" } else { "风险" }
    })
}

fn cohort_analysis(input: &Value) -> Value {
    let unit = s(input, "cohort_unit", "week");
    let label = if unit == "month" { "月" } else { "周" };
    let new_customers = i(input, "new_customers", 120).max(1);
    let base = [1.0, 0.35, 0.22, 0.15, 0.10, 0.07];
    let cohorts = (0..4)
        .map(|idx| {
            let acquired = (new_customers as f64 * (0.9_f64).powi(idx)).round() as i64;
            let curve = base
                .iter()
                .take(6 - idx as usize)
                .enumerate()
                .map(|(period, rate)| json!({format!("第{period}{label}"): format!("{:.1}% ({}人)", rate * 100.0, (acquired as f64 * rate).round() as i64)}))
                .collect::<Vec<_>>();
            json!({"队列": format!("第{}{label}队列", idx + 1), "获取新客": acquired, "留存曲线": curve})
        })
        .collect::<Vec<_>>();
    json!({
        "数据来源": "指数留存模型",
        "队列留存矩阵": cohorts,
        "关键流失节点": [{"节点": format!("第1{label}后"), "流失率": "约65%", "原因": "首次体验不满意或无复购触点"}],
        "关键发现": {"首期留存率": "约35%", "最高价值窗口": format!("首次购买后第1-2{label}")}
    })
}

fn ab_test_analyzer(input: &Value) -> Result<Value, SkillError> {
    let ctrl_n = required_i(input, "control_visitors")?;
    let ctrl_c = required_i(input, "control_conversions")?;
    let test_n = required_i(input, "test_visitors")?;
    let test_c = required_i(input, "test_conversions")?;
    if ctrl_n <= 0 || test_n <= 0 {
        return Err(SkillError::invalid_parameter(
            "visitors",
            "positive integer",
        ));
    }
    if ctrl_c < 0 || test_c < 0 || ctrl_c > ctrl_n || test_c > test_n {
        return Err(SkillError::invalid_parameter(
            "conversions",
            "0 <= conversions <= visitors",
        ));
    }
    let ctrl_rate = ctrl_c as f64 / ctrl_n as f64;
    let test_rate = test_c as f64 / test_n as f64;
    let pooled = (ctrl_c + test_c) as f64 / (ctrl_n + test_n) as f64;
    let se = (pooled * (1.0 - pooled) * (1.0 / ctrl_n as f64 + 1.0 / test_n as f64)).sqrt();
    let z = if se > 0.0 {
        (test_rate - ctrl_rate) / se
    } else {
        0.0
    };
    let confidence = normal_confidence(z.abs());
    json_ok(json!({
        "测试名称": s(input, "test_name", "A/B测试"),
        "指标": s(input, "metric_name", "转化率"),
        "对照组转化率": percent(ctrl_rate * 100.0, 2),
        "实验组转化率": percent(test_rate * 100.0, 2),
        "相对提升": signed_percent(pct(test_rate - ctrl_rate, ctrl_rate), 2),
        "Z值": round4(z),
        "置信度": percent(confidence * 100.0, 2),
        "结论": if confidence >= 0.95 { "差异显著，可考虑推全" } else if confidence >= 0.90 { "接近显著，建议扩大样本" } else { "差异不显著，继续观察" }
    }))
}

fn refund_decomposition(input: &Value) -> Value {
    let items = input
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return deferred(
            "data_refund_decomposition",
            "需要退款金额、退款率或渠道退款明细",
            input,
        );
    }
    let mut rows = items
        .iter()
        .map(|item| {
            let amount = n(
                item,
                "refund_amount",
                n(item, "gmv", 0.0) * n(item, "refund_rate", 0.03),
            );
            (amount, item)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    let total = rows.iter().map(|(amount, _)| *amount).sum::<f64>().max(1.0);
    let mut cumulative = 0.0;
    let top_n = i(input, "top_n", 5).max(1) as usize;
    let pareto = rows
        .iter()
        .take(top_n)
        .map(|(amount, item)| {
            let share = *amount / total * 100.0;
            cumulative += share;
            json!({"项目": s(item, "name", "未知"), "退款金额": round2(*amount), "占总退款比": percent(share, 1), "累计占比": percent(cumulative.min(100.0), 1), "是否80%内": cumulative <= 80.0})
        })
        .collect::<Vec<_>>();
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": "退款拆解分析完成",
        "总退款估算": round2(total),
        "帕累托分析": pareto,
        "整改建议": ["优先处理累计80%内的问题项", "复核详情页描述、质检和物流包装"]
    })
}

fn seasonal_decompose(input: &Value) -> Value {
    let series = nums(input, "values");
    let period = i(input, "period", 7).max(2) as usize;
    if series.len() < period * 2 {
        return json!({"has_data": false, "提示": format!("数据点不足（需≥{}个），当前仅{}个", period * 2, series.len())});
    }
    let ma = moving_average(&series, period);
    let trend_start = ma.first().copied().unwrap_or(series[0]);
    let trend_end = ma
        .last()
        .copied()
        .unwrap_or(*series.last().unwrap_or(&series[0]));
    let trend_growth = pct(trend_end - trend_start, trend_start);
    let mut indices = Vec::new();
    for p in 0..period {
        let vals = series
            .iter()
            .enumerate()
            .filter_map(|(idx, value)| (idx % period == p).then_some(*value))
            .collect::<Vec<_>>();
        indices.push(average(&vals) / average(&series).max(1e-9));
    }
    let strength = stddev(&indices) * 100.0;
    json!({
        "has_data": true,
        "指标": metric_display(&s(input, "metric", "gmv")),
        "季节周期": format!("{period}天"),
        "趋势分析": {"趋势起点": round2(trend_start), "趋势终点": round2(trend_end), "趋势增长率": signed_percent(trend_growth, 2)},
        "季节性分析": {"季节性强度": percent(strength, 1), "季节指数": indices.iter().map(|v| round4(*v)).collect::<Vec<_>>()},
        "运营建议": if trend_growth >= 0.0 { "趋势向上，峰值周期前加大备货和投放" } else { "趋势下行，优先排查流量、价格和竞品冲击" }
    })
}

fn price_elasticity(input: &Value) -> Value {
    let points = input
        .get("price_points")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| {
            vec![
                json!({"price": 90.0, "quantity": 560.0}),
                json!({"price": 110.0, "quantity": 440.0}),
            ]
        });
    let pairs = points
        .iter()
        .filter_map(|point| {
            let price = n(point, "price", 0.0);
            let qty = n(point, "quantity", 0.0);
            (price > 0.0 && qty > 0.0).then_some((price, qty))
        })
        .collect::<Vec<_>>();
    if pairs.len() < 2 {
        return json!({"error": "至少需要2个有效 price_points"});
    }
    let mut elasticities = Vec::new();
    for pair in pairs.windows(2) {
        let (p1, q1) = pair[0];
        let (p2, q2) = pair[1];
        let pct_q = (q2 - q1) / ((q1 + q2) / 2.0);
        let pct_p = (p2 - p1) / ((p1 + p2) / 2.0);
        if pct_p.abs() > 1e-9 {
            elasticities.push(pct_q / pct_p);
        }
    }
    let e = average(&elasticities);
    let cost = n(input, "cost", pairs[0].0 * 0.45);
    let current = n(
        input,
        "current_price",
        pairs.last().map(|(p, _)| *p).unwrap_or(100.0),
    );
    let optimal = if e < -1.0 {
        cost * e / (e + 1.0)
    } else {
        current
    };
    json!({
        "价格弹性系数": round4(e),
        "敏感度分类": if e.abs() > 1.5 { "高弹性" } else if e.abs() > 0.8 { "中弹性" } else { "低弹性" },
        "当前售价": round2(current),
        "Lerner最优价参考": round2(optimal.max(cost * 1.05)),
        "建议": if e < -1.0 { "需求对价格敏感，降价促量需同步核算利润" } else { "需求弹性较低，可测试小幅提价" }
    })
}

fn demand_forecast(input: &Value) -> Value {
    let values = nums(input, "values");
    if values.len() < 3 {
        return json!({"error": "数据不足", "hint": "请提供 values 至少3个数据点"});
    }
    let horizons = ints(input, "forecast_days", vec![7, 14, 30]);
    let alphas = [0.2, 0.4, 0.6, 0.8];
    let mut best_alpha = 0.2;
    let mut best_mape = f64::INFINITY;
    for alpha in alphas {
        let preds = ses_fitted(&values, alpha);
        let score = mape(&values[1..], &preds);
        if score < best_mape {
            best_mape = score;
            best_alpha = alpha;
        }
    }
    let level = ses_level(&values, best_alpha);
    let sigma = stddev(&values);
    let forecasts = horizons
        .into_iter()
        .map(|days| json!({"未来天数": days, "预测总量": round2(level * days as f64), "日均预测": round2(level), "置信区间": [round2((level - 1.96 * sigma).max(0.0)), round2(level + 1.96 * sigma)]}))
        .collect::<Vec<_>>();
    json!({
        "指标": metric_display(&s(input, "metric", "orders")),
        "历史数据点数": values.len(),
        "最优alpha": best_alpha,
        "MAPE": percent(best_mape, 2),
        "预测结果": forecasts,
        "备货建议": format!("按日均{}的预测水平备货，并预留约{}安全缓冲", round2(level), round2(1.65 * sigma))
    })
}

fn comprehensive_diagnosis(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let focus = normalize_focus(&s(input, "focus", "all"));
    if !has_comprehensive_diagnosis_input(input) {
        return comprehensive_deferred(
            "需要至少提供经营指标、漏斗数据、时序数据、渠道数据、退款明细、客户指标或LTV输入中的一类",
            input,
        );
    }

    let parts = vec![
        diagnosis_part("经营看板", "data_dashboard", data_dashboard(input), input),
        diagnosis_part(
            "漏斗转化",
            "data_funnel_analysis",
            funnel_analysis(input),
            input,
        ),
        diagnosis_part(
            "异常检测",
            "data_anomaly_diagnosis",
            anomaly_diagnosis(input),
            input,
        ),
        diagnosis_part(
            "客户分层",
            "data_customer_segmentation",
            customer_segmentation(input),
            input,
        ),
        diagnosis_part(
            "退款根因",
            "data_refund_decomposition",
            refund_decomposition(input),
            input,
        ),
        diagnosis_part(
            "需求预测",
            "data_demand_forecast",
            demand_forecast(input),
            input,
        ),
        diagnosis_part(
            "渠道归因",
            "data_attribution_analysis",
            attribution_analysis(input),
            input,
        ),
        diagnosis_part(
            "LTV健康度",
            "data_ltv_calculator",
            ltv_calculator(input),
            input,
        ),
    ];

    let user_success_count = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .count();
    if user_success_count == 0 {
        return comprehensive_deferred(
            "已有输入不足以形成真实综合诊断；请补充至少一类可计算业务数据",
            input,
        );
    }

    let success_count = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "success")
        .count();
    let deferred_count = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .count();
    let default_count = parts
        .iter()
        .filter(|part| {
            matches!(
                diagnosis_readiness(part).as_str(),
                "calculated_from_defaults" | "sample_only"
            )
        })
        .count();
    let has_partial_data = deferred_count > 0 || default_count > 0;
    let health_score = comprehensive_health_score(&parts);
    let grade = diagnosis_grade(health_score);
    let findings = comprehensive_findings(&parts);
    let risks = comprehensive_risks(&parts);
    let opportunities = comprehensive_opportunities(&parts, &focus);
    let action_plan = comprehensive_action_plan(&parts, &focus);
    let missing_data = comprehensive_missing_data(&parts);

    json!({
        "status": "success",
        "has_data": true,
        "has_partial_data": has_partial_data,
        "adapter_status": if has_partial_data { "partial_manual_input" } else { "manual_input" },
        "data_readiness": if has_partial_data { "partial_user_input" } else { "user_provided" },
        "summary": format!("综合业务诊断完成，健康评分{}分，等级{}", health_score, grade),
        "分析天数": days,
        "重点方向": focus,
        "分析口径": focus_instruction(&focus),
        "数据来源": "基于本次用户显式输入和新产品 Rust Agent 数据 skill 编排；未直连淘宝、抖音、千川或数仓实时数据",
        "诊断覆盖": {
            "成功分项": success_count,
            "真实输入成功分项": user_success_count,
            "缺数据分项": deferred_count,
            "默认估算分项": default_count
        },
        "整体健康评分": {
            "总分": health_score,
            "等级": grade,
            "评分口径": "优先采用经营看板评分；无完整看板时按真实分项覆盖、风险数量和缺失数据进行规则评分"
        },
        "综合诊断报告": build_diagnosis_report(DiagnosisReportDraft {
            days,
            focus: &focus,
            score: health_score,
            grade,
            findings: &findings,
            risks: &risks,
            opportunities: &opportunities,
            actions: &action_plan,
        }),
        "分项诊断": parts,
        "核心发现": findings,
        "风险预警": risks,
        "增长机会": opportunities,
        "7天行动计划": action_plan,
        "缺失数据": missing_data,
        "治理说明": "缺失分项以 deferred 明示，默认估算分项以 data_readiness 标记；前端不能展示为已读取平台实时数据。"
    })
}

fn comprehensive_deferred(reason: &str, input: &Value) -> Value {
    json!({
        "status": "deferred",
        "skill": "data_comprehensive_diagnosis",
        "has_data": false,
        "adapter_status": "user_input_required",
        "data_readiness": "missing_user_input",
        "deferred_reason": reason,
        "summary": data_summary("data_comprehensive_diagnosis"),
        "received_params": input,
        "expected_inputs": [
            "经营指标：gmv/orders/visitors/clicks/ad_spend/refund_amount/gross_profit",
            "漏斗数据：impressions/clicks/add_to_cart/orders/payments",
            "时序数据：values",
            "渠道数据：channels",
            "退款明细：items",
            "客户或LTV指标：total_customers/cac/avg_order_value/purchase_frequency"
        ],
        "side_effects": "none"
    })
}

fn diagnosis_part(module: &str, skill: &str, mut output: Value, input: &Value) -> Value {
    normalize_data_output(skill, input, &mut output);
    let status = value_string(&output, "status", "success");
    let data_readiness = value_string(&output, "data_readiness", "user_provided");
    let adapter_status = value_string(&output, "adapter_status", "manual_input");
    let has_data = output
        .get("has_data")
        .and_then(Value::as_bool)
        .unwrap_or(status == "success");
    let conclusion = diagnosis_conclusion(skill, &output);
    let evidence = diagnosis_evidence(skill, &output);

    json!({
        "模块": module,
        "skill": skill,
        "status": status,
        "has_data": has_data,
        "data_readiness": data_readiness,
        "adapter_status": adapter_status,
        "结论": conclusion,
        "关键证据": evidence,
        "output": output
    })
}

fn has_comprehensive_diagnosis_input(input: &Value) -> bool {
    has_any_input(
        input,
        &[
            "gmv",
            "revenue",
            "previous_gmv",
            "orders",
            "order_count",
            "previous_orders",
            "visitors",
            "uv",
            "previous_visitors",
            "clicks",
            "add_to_cart",
            "payments",
            "impressions",
            "ad_spend",
            "ad_cost",
            "refund_amount",
            "refunds",
            "gross_profit",
            "profit",
            "target_gmv",
            "goal_gmv",
            "total_customers",
            "avg_order_value",
            "avg_orders_per_customer",
            "recency_days",
            "cac",
            "purchase_frequency",
            "gross_margin",
            "monthly_retention",
        ],
    ) || nums(input, "values").len() >= 3
        || !input_array_missing_or_empty(input, "channels")
        || !input_array_missing_or_empty(input, "items")
}

fn normalize_focus(focus: &str) -> String {
    match focus {
        "growth" | "profit" | "retention" | "risk" | "all" => focus.to_owned(),
        _ => "all".to_owned(),
    }
}

fn focus_instruction(focus: &str) -> &'static str {
    match focus {
        "growth" => "重点聚焦增长机会、流量效率和转化改善",
        "profit" => "重点聚焦利润结构、投放效率和成本优化",
        "retention" => "重点聚焦客户留存、复购和LTV健康度",
        "risk" => "重点聚焦异常、退款、毛利和经营风险预警",
        _ => "全面均衡诊断，兼顾增长、利润、留存和风险",
    }
}

fn diagnosis_status(part: &Value) -> String {
    value_string(part, "status", "success")
}

fn diagnosis_readiness(part: &Value) -> String {
    value_string(part, "data_readiness", "user_provided")
}

fn diagnosis_is_user_success(part: &Value) -> bool {
    diagnosis_status(part) == "success"
        && part
            .get("has_data")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        && diagnosis_readiness(part) == "user_provided"
}

fn diagnosis_conclusion(skill: &str, output: &Value) -> String {
    let status = value_string(output, "status", "success");
    if status == "deferred" {
        return format!("缺数据：{}", data_deferred_reason_from_output(output));
    }
    if let Some(summary) = output.get("summary").and_then(Value::as_str) {
        return summary.to_owned();
    }
    match skill {
        "data_dashboard" => {
            let score = value_pointer_number(output, "/经营健康度/总分")
                .map(|value| format!("{}分", round2(value)))
                .unwrap_or_else(|| "未评分".to_owned());
            let grade = value_pointer_string(output, "/经营健康度/等级", "未知");
            format!("经营健康度{score}，等级{grade}")
        }
        "data_anomaly_diagnosis" => {
            let count = output
                .get("异常点数")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            format!("发现{count}个异常点")
        }
        "data_customer_segmentation" => "客户分层已生成".to_owned(),
        "data_ltv_calculator" => {
            format!("LTV健康度{}", value_string(output, "健康度", "待评估"))
        }
        _ => data_summary(skill).to_owned(),
    }
}

fn diagnosis_evidence(skill: &str, output: &Value) -> Value {
    match skill {
        "data_dashboard" => json!({
            "健康度": output.get("经营健康度").cloned().unwrap_or(Value::Null),
            "预警数": output.get("预警").and_then(Value::as_array).map(Vec::len).unwrap_or_default()
        }),
        "data_funnel_analysis" => json!({
            "瓶颈环节": output.get("瓶颈环节").cloned().unwrap_or(Value::Null),
            "整体转化率": output.get("整体转化率").cloned().unwrap_or(Value::Null)
        }),
        "data_anomaly_diagnosis" => json!({
            "异常点数": output.get("异常点数").cloned().unwrap_or(Value::Null),
            "样本数": output.get("样本数").cloned().unwrap_or(Value::Null)
        }),
        "data_refund_decomposition" => json!({
            "总退款估算": output.get("总退款估算").cloned().unwrap_or(Value::Null),
            "帕累托分析": output.get("帕累托分析").cloned().unwrap_or(Value::Null)
        }),
        "data_demand_forecast" => json!({
            "预测结果": output.get("预测结果").cloned().unwrap_or(Value::Null),
            "备货建议": output.get("备货建议").cloned().unwrap_or(Value::Null)
        }),
        "data_attribution_analysis" => json!({
            "渠道数": output.get("渠道数").cloned().unwrap_or(Value::Null),
            "预算再分配建议": output.get("预算再分配建议").cloned().unwrap_or(Value::Null)
        }),
        "data_ltv_calculator" => json!({
            "LTV": output.get("LTV").cloned().unwrap_or(Value::Null),
            "LTV/CAC": output.get("LTV/CAC").cloned().unwrap_or(Value::Null),
            "健康度": output.get("健康度").cloned().unwrap_or(Value::Null)
        }),
        _ => json!({"summary": output.get("summary").cloned().unwrap_or(Value::Null)}),
    }
}

fn comprehensive_health_score(parts: &[Value]) -> i64 {
    if let Some(score) = parts
        .iter()
        .find(|part| {
            part.get("skill").and_then(Value::as_str) == Some("data_dashboard")
                && diagnosis_is_user_success(part)
        })
        .and_then(|part| part.get("output"))
        .and_then(|output| value_pointer_number(output, "/经营健康度/总分"))
    {
        return score.round().clamp(0.0, 100.0) as i64;
    }

    let user_success = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .count() as i64;
    let deferred = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .count() as i64;
    let risk_count = comprehensive_risks(parts).len() as i64;
    (62 + user_success * 5 - deferred * 3 - risk_count * 2).clamp(35, 92)
}

fn diagnosis_grade(score: i64) -> &'static str {
    if score >= 80 {
        "健康"
    } else if score >= 60 {
        "需关注"
    } else {
        "风险"
    }
}

fn comprehensive_findings(parts: &[Value]) -> Vec<String> {
    let mut findings = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .filter_map(|part| {
            let module = part.get("模块").and_then(Value::as_str)?;
            let conclusion = part.get("结论").and_then(Value::as_str)?;
            Some(format!("{module}：{conclusion}"))
        })
        .take(6)
        .collect::<Vec<_>>();
    if findings.is_empty() {
        findings.push("当前输入不足，尚未形成可引用的核心发现".to_owned());
    }
    findings
}

fn comprehensive_risks(parts: &[Value]) -> Vec<String> {
    let mut risks = Vec::new();
    for part in parts {
        if !diagnosis_is_user_success(part) {
            continue;
        }
        let skill = part
            .get("skill")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(output) = part.get("output") else {
            continue;
        };
        match skill {
            "data_dashboard" => {
                if let Some(alerts) = output.get("预警").and_then(Value::as_array) {
                    for alert in alerts.iter().take(3) {
                        let level = alert.get("level").and_then(Value::as_str).unwrap_or("");
                        if level == "warning" {
                            risks.push(value_string(alert, "message", "经营指标触发预警"));
                        }
                    }
                }
            }
            "data_anomaly_diagnosis" => {
                let count = output
                    .get("异常点数")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                if count > 0 {
                    risks.push(format!(
                        "时序数据发现{count}个异常点，需要排查活动、投放或供给变化。"
                    ));
                }
            }
            "data_refund_decomposition" if diagnosis_status(part) == "success" => {
                if let Some(first) = output
                    .get("帕累托分析")
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                {
                    risks.push(format!(
                        "退款集中在{}，应优先处理头部售后原因。",
                        value_string(first, "项目", "关键问题项")
                    ));
                }
            }
            _ => {}
        }
    }
    if risks.is_empty() {
        risks.push("未发现明确高优先级风险；仍需持续补充平台实时数据进行复核。".to_owned());
    }
    risks.into_iter().take(5).collect()
}

fn comprehensive_opportunities(parts: &[Value], focus: &str) -> Vec<String> {
    let mut opportunities = Vec::new();
    for part in parts {
        if !diagnosis_is_user_success(part) {
            continue;
        }
        let skill = part
            .get("skill")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(output) = part.get("output") else {
            continue;
        };
        match skill {
            "data_funnel_analysis" if diagnosis_status(part) == "success" => {
                opportunities.push(format!(
                    "围绕{}做转化专项优化：{}。",
                    value_string(output, "瓶颈环节", "当前瓶颈"),
                    value_string(output, "优化建议", "复核流量和转化链路")
                ));
            }
            "data_attribution_analysis" if diagnosis_status(part) == "success" => {
                opportunities.push(value_string(
                    output,
                    "预算再分配建议",
                    "按归因贡献重排渠道预算",
                ));
            }
            "data_demand_forecast" if diagnosis_status(part) == "success" => {
                opportunities.push(value_string(output, "备货建议", "按预测需求安排库存和补货"));
            }
            "data_ltv_calculator" if diagnosis_status(part) == "success" => {
                opportunities.push(format!(
                    "以LTV/CAC={}为复购和获客约束，优先提升高价值客群复购。",
                    output
                        .get("LTV/CAC")
                        .map(value_to_short_string)
                        .unwrap_or_else(|| "待评估".to_owned())
                ));
            }
            _ => {}
        }
    }
    if opportunities.is_empty() {
        opportunities.push(match focus {
            "profit" => "先补齐毛利、广告花费和退款结构，再定位利润优化机会。".to_owned(),
            "retention" => "先补齐客户、复购和LTV指标，再定位留存提升机会。".to_owned(),
            "risk" => "先补齐时序、退款和毛利指标，再定位经营风险。".to_owned(),
            _ => "先补齐流量、转化和渠道数据，再定位增长机会。".to_owned(),
        });
    }
    opportunities.into_iter().take(5).collect()
}

fn comprehensive_action_plan(parts: &[Value], focus: &str) -> Vec<Value> {
    let mut actions = Vec::new();
    let missing = comprehensive_missing_data(parts);
    if !missing.is_empty() {
        actions.push(json!({
            "优先级": "P0",
            "动作": "补齐缺失数据并重新运行综合诊断",
            "依据": missing.iter().take(3).cloned().collect::<Vec<_>>()
        }));
    }
    for (idx, risk) in comprehensive_risks(parts).into_iter().take(2).enumerate() {
        actions.push(json!({
            "优先级": format!("P{}", idx + 1),
            "动作": format!("处理风险：{risk}"),
            "依据": "风险预警"
        }));
    }
    for (idx, opportunity) in comprehensive_opportunities(parts, focus)
        .into_iter()
        .take(2)
        .enumerate()
    {
        actions.push(json!({
            "优先级": format!("P{}", idx + 3),
            "动作": opportunity,
            "依据": "增长机会"
        }));
    }
    actions.into_iter().take(5).collect()
}

fn comprehensive_missing_data(parts: &[Value]) -> Vec<String> {
    parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .filter_map(|part| {
            let module = part.get("模块").and_then(Value::as_str)?;
            let output = part.get("output")?;
            Some(format!(
                "{module}：{}",
                data_deferred_reason_from_output(output)
            ))
        })
        .collect()
}

struct DiagnosisReportDraft<'a> {
    days: i64,
    focus: &'a str,
    score: i64,
    grade: &'a str,
    findings: &'a [String],
    risks: &'a [String],
    opportunities: &'a [String],
    actions: &'a [Value],
}

fn build_diagnosis_report(draft: DiagnosisReportDraft<'_>) -> String {
    let mut report = format!(
        "【诊断基于{}天数据】\n整体健康评分：{}分（{}）。\n分析重点：{}。\n\n",
        draft.days,
        draft.score,
        draft.grade,
        focus_instruction(draft.focus)
    );
    report.push_str("核心发现：\n");
    for finding in draft.findings.iter().take(5) {
        report.push_str(&format!("- {finding}\n"));
    }
    report.push_str("\n风险预警：\n");
    for risk in draft.risks.iter().take(3) {
        report.push_str(&format!("- {risk}\n"));
    }
    report.push_str("\n增长机会：\n");
    for opportunity in draft.opportunities.iter().take(3) {
        report.push_str(&format!("- {opportunity}\n"));
    }
    report.push_str("\n7天行动计划：\n");
    for action in draft.actions.iter().take(5) {
        report.push_str(&format!(
            "- {}：{}\n",
            value_string(action, "优先级", "P?"),
            value_string(action, "动作", "待补充动作")
        ));
    }
    report
}

fn data_deferred_reason_from_output(output: &Value) -> String {
    output
        .as_object()
        .map(data_deferred_reason)
        .unwrap_or_else(|| "需要补充用户输入数据".to_owned())
}

fn value_string(input: &Value, key: &str, default: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

fn value_pointer_string<'a>(input: &'a Value, path: &str, default: &'a str) -> &'a str {
    input
        .pointer(path)
        .and_then(Value::as_str)
        .unwrap_or(default)
}

fn value_pointer_number(input: &Value, path: &str) -> Option<f64> {
    input.pointer(path).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_i64().map(|number| number as f64))
    })
}

fn value_to_short_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn number_from_any(input: &Value, keys: &[&str], default: f64) -> (f64, bool) {
    for key in keys {
        if let Some(value) = input.get(*key).and_then(Value::as_f64) {
            return (value.max(0.0), true);
        }
        if let Some(value) = input.get(*key).and_then(Value::as_i64) {
            return ((value as f64).max(0.0), true);
        }
    }
    (default.max(0.0), false)
}

#[derive(Debug, Clone)]
struct AttributionChannel {
    name: String,
    gmv: f64,
    uv: f64,
    conversion_rate: f64,
    ad_spend: f64,
}

fn normalized_rate(value: f64) -> f64 {
    if value > 1.0 {
        value / 100.0
    } else {
        value.max(0.0)
    }
}

fn normalize(values: Vec<f64>) -> Vec<f64> {
    let total = values.iter().sum::<f64>();
    if total.abs() < 1e-9 {
        let len = values.len().max(1);
        vec![1.0 / len as f64; len]
    } else {
        values.into_iter().map(|value| value / total).collect()
    }
}

fn one_hot(len: usize, active_idx: usize) -> Vec<f64> {
    (0..len)
        .map(|idx| if idx == active_idx { 1.0 } else { 0.0 })
        .collect()
}

fn position_based_weights(len: usize) -> Vec<f64> {
    match len {
        0 => Vec::new(),
        1 => vec![1.0],
        2 => vec![0.5, 0.5],
        3 => vec![0.4, 0.2, 0.4],
        _ => {
            let middle = 0.20 / (len - 2) as f64;
            let mut weights = Vec::with_capacity(len);
            weights.push(0.4);
            weights.extend(std::iter::repeat_n(middle, len - 2));
            weights.push(0.4);
            normalize(weights)
        }
    }
}

fn attribution_models_to_show(model: &str) -> Vec<&'static str> {
    match model {
        "last_touch" => vec!["last_touch"],
        "linear" => vec!["linear"],
        "time_decay" => vec!["time_decay"],
        "position" => vec!["position"],
        _ => vec!["last_touch", "linear", "time_decay", "position"],
    }
}

fn pct_delta(current: f64, previous: f64) -> f64 {
    pct(current - previous, previous.max(1.0))
}

fn metric_card(label: &str, value: f64, unit: &str, delta: f64, note: &str) -> Value {
    json!({
        "指标": label,
        "数值": value,
        "单位": unit,
        "变化": signed_percent(delta, 2),
        "状态": if delta >= 0.0 { "改善" } else if delta > -5.0 { "轻微下滑" } else { "需关注" },
        "说明": note
    })
}

fn dashboard_health_score(
    gmv_growth: f64,
    conversion_rate: f64,
    ad_roi: f64,
    refund_rate: f64,
    gross_margin: f64,
    target_achievement: f64,
) -> i64 {
    let score = 50.0
        + gmv_growth.clamp(-20.0, 20.0) * 0.6
        + (conversion_rate - 3.0).clamp(-3.0, 8.0) * 2.0
        + (ad_roi - 2.0).clamp(-2.0, 6.0) * 3.0
        + (gross_margin - 25.0).clamp(-20.0, 30.0) * 0.5
        + (target_achievement - 80.0).clamp(-40.0, 40.0) * 0.2
        - (refund_rate - 4.0).clamp(0.0, 12.0) * 2.0;
    score.round().clamp(0.0, 100.0) as i64
}

fn growth_summary(gmv_growth: f64, order_growth: f64, visitor_growth: f64) -> &'static str {
    if gmv_growth >= 5.0 && order_growth >= 3.0 {
        "收入和成交同步增长，可继续放大有效渠道"
    } else if visitor_growth >= 5.0 && order_growth < 0.0 {
        "流量增长但成交未跟上，重点排查转化承接"
    } else if gmv_growth < -5.0 {
        "收入动能走弱，优先定位流量、订单和客单价变化"
    } else {
        "增长基本平稳，建议持续观察渠道结构"
    }
}

fn efficiency_summary(conversion_rate: f64, ad_roi: f64, aov: f64) -> String {
    if conversion_rate < 2.0 {
        "转化效率偏弱，先优化详情页和首屏卖点".to_string()
    } else if ad_roi < 2.0 {
        "投放效率偏弱，先收缩低ROI计划".to_string()
    } else {
        format!("效率稳定，当前客单价约{}", round2(aov))
    }
}

fn risk_summary(refund_rate: f64, gross_margin: f64) -> &'static str {
    if refund_rate > 6.0 {
        "售后风险偏高，优先拆解退款原因"
    } else if gross_margin < 20.0 {
        "利润安全垫不足，需复核成本和优惠"
    } else {
        "退款和毛利暂未触发高风险阈值"
    }
}

fn dashboard_actions(
    gmv_growth: f64,
    conversion_rate: f64,
    ad_roi: f64,
    refund_rate: f64,
    gross_margin: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if gmv_growth < -5.0 {
        actions.push("拆分GMV下滑来源：流量、转化、客单价、退款");
    }
    if conversion_rate < 2.0 {
        actions.push("复核详情页首屏、价格锚点、评价和客服响应");
    }
    if ad_roi < 2.0 {
        actions.push("暂停低ROI计划，保留高转化素材继续A/B测试");
    }
    if refund_rate > 6.0 {
        actions.push("按退款原因做帕累托拆解，优先处理累计80%问题项");
    }
    if gross_margin < 20.0 {
        actions.push("复核进货成本、平台扣点、优惠券和包邮策略");
    }
    if actions.is_empty() {
        actions.push("保持当前节奏，下一轮重点观察渠道结构和复购");
    }
    actions
}

fn json_ok(value: Value) -> Result<Value, SkillError> {
    Ok(value)
}

fn required_string(input: &Value, key: &str) -> Result<String, SkillError> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SkillError::missing_parameter(key))
}

fn required_i(input: &Value, key: &str) -> Result<i64, SkillError> {
    input
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| SkillError::missing_parameter(key))
}

fn s(input: &Value, key: &str, default: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

fn i(input: &Value, key: &str, default: i64) -> i64 {
    input.get(key).and_then(Value::as_i64).unwrap_or(default)
}

fn n(input: &Value, key: &str, default: f64) -> f64 {
    input.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn nums(input: &Value, key: &str) -> Vec<f64> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

fn has_any_input(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        input.get(*key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => true,
        })
    })
}

fn input_array_missing_or_empty(input: &Value, key: &str) -> bool {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::is_empty)
        .unwrap_or(true)
}

fn ints(input: &Value, key: &str, default: Vec<i64>) -> Vec<i64> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_i64).collect())
        .filter(|values: &Vec<i64>| !values.is_empty())
        .unwrap_or(default)
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn stddev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = average(values);
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

fn linear_fit(values: &[f64]) -> (f64, f64) {
    let len = values.len() as f64;
    let x_mean = (len - 1.0) / 2.0;
    let y_mean = average(values);
    let numerator = values
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx as f64 - x_mean) * (value - y_mean))
        .sum::<f64>();
    let denominator = (0..values.len())
        .map(|idx| (idx as f64 - x_mean).powi(2))
        .sum::<f64>();
    let slope = if denominator.abs() > 1e-9 {
        numerator / denominator
    } else {
        0.0
    };
    (slope, y_mean - slope * x_mean)
}

fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    (0..values.len())
        .map(|idx| {
            let start = (idx + 1).saturating_sub(window);
            round2(average(&values[start..=idx]))
        })
        .collect()
}

fn avg_growth(values: &[f64]) -> f64 {
    let rates = values
        .windows(2)
        .filter_map(|pair| {
            (pair[0].abs() > 1e-9).then_some((pair[1] - pair[0]) / pair[0].abs() * 100.0)
        })
        .collect::<Vec<_>>();
    average(&rates)
}

fn ses_level(values: &[f64], alpha: f64) -> f64 {
    let mut level = values[0];
    for value in &values[1..] {
        level = alpha * *value + (1.0 - alpha) * level;
    }
    level
}

fn ses_fitted(values: &[f64], alpha: f64) -> Vec<f64> {
    let mut level = values[0];
    let mut fitted = Vec::with_capacity(values.len().saturating_sub(1));
    for value in &values[1..] {
        fitted.push(level);
        level = alpha * *value + (1.0 - alpha) * level;
    }
    fitted
}

fn mape(actual: &[f64], predicted: &[f64]) -> f64 {
    let errors = actual
        .iter()
        .zip(predicted)
        .filter_map(|(actual, predicted)| {
            (actual.abs() > 1e-9).then_some(((actual - predicted) / actual).abs() * 100.0)
        })
        .collect::<Vec<_>>();
    average(&errors)
}

fn normal_confidence(z_abs: f64) -> f64 {
    if z_abs >= 2.576 {
        0.99
    } else if z_abs >= 1.96 {
        0.95
    } else if z_abs >= 1.645 {
        0.90
    } else {
        (0.5 + z_abs / 3.29).min(0.89)
    }
}

fn metric_display(metric: &str) -> &str {
    match metric {
        "gmv" => "GMV",
        "orders" => "订单数",
        "uv" => "访客数(UV)",
        "conversion_rate" => "转化率",
        "ad_spend" => "广告花费",
        _ => metric,
    }
}

fn pct(num: f64, den: f64) -> f64 {
    if den.abs() < 1e-9 {
        0.0
    } else {
        num / den * 100.0
    }
}

fn percent(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}%")
}

fn signed_percent(value: f64, decimals: usize) -> String {
    format!("{value:+.decimals$}%")
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn query_store_metrics_marks_success_for_explicit_metrics() {
        let outcome = execute(
            "query_store_metrics",
            json!({
                "days": 14,
                "platform": "douyin",
                "gmv": 180000.0,
                "previous_gmv": 150000.0,
                "orders": 1800,
                "previous_orders": 1500,
                "visitors": 36000,
                "previous_visitors": 33000,
                "clicks": 7200,
                "ad_spend": 36000.0,
                "refund_amount": 5400.0,
                "gross_profit": 54000.0
            }),
            &SkillContext::default(),
        )
        .await
        .expect("query_store_metrics should run from explicit metrics");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("explicit_metrics")
        );
        assert_eq!(output.pointer("/totals/gmv"), Some(&json!(180000.0)));
        assert_eq!(
            output.pointer("/derived_metrics/conversion_rate"),
            Some(&json!("5.00%"))
        );
        assert!(output.pointer("/week_over_week/gmv/change_rate").is_some());
    }

    #[tokio::test]
    async fn query_store_metrics_defers_without_metrics() {
        let outcome = execute(
            "query_store_metrics",
            json!({"days": 30}),
            &SkillContext::default(),
        )
        .await
        .expect("query_store_metrics should return structured deferred");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(false));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("store_metrics_required")
        );
        assert!(
            output
                .get("deferred_reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("不会伪造平台实时数据")
        );
    }

    #[tokio::test]
    async fn query_store_metrics_filters_platform_items() {
        let outcome = execute(
            "query_store_metrics",
            json!({
                "platform": "douyin",
                "platforms": [
                    {"platform": "douyin", "gmv": 120000.0, "orders": 1000, "uv": 20000, "clicks": 5000},
                    {"platform": "taobao", "gmv": 80000.0, "orders": 700, "uv": 15000, "clicks": 2600}
                ]
            }),
            &SkillContext::default(),
        )
        .await
        .expect("query_store_metrics should filter platform rows");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.pointer("/totals/gmv"), Some(&json!(120000.0)));
        assert_eq!(
            output
                .get("platforms")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1)
        );
    }

    #[tokio::test]
    async fn query_product_metrics_marks_success_for_explicit_metrics() {
        let outcome = execute(
            "query_product_metrics",
            json!({
                "product_ref": "product:11111111-1111-1111-1111-111111111111",
                "product_id": "prod-001",
                "product_name": "冷萃咖啡",
                "sku_id": "sku-a",
                "platform": "抖音",
                "gmv": 96000.0,
                "previous_gmv": 80000.0,
                "orders": 1200,
                "visitors": 24000,
                "clicks": 4800,
                "ad_spend": 24000.0,
                "refund_amount": 2800.0,
                "gross_profit": 32000.0,
                "inventory": 8,
                "stockout_count": 1
            }),
            &SkillContext::default(),
        )
        .await
        .expect("query_product_metrics should run from explicit metrics");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("explicit_product_metrics")
        );
        assert_eq!(output.pointer("/totals/gmv"), Some(&json!(96000.0)));
        assert_eq!(
            output.pointer("/query/product_ref"),
            Some(&json!("product:11111111-1111-1111-1111-111111111111"))
        );
        assert_eq!(
            output.pointer("/query/product_id"),
            Some(&json!("prod-001"))
        );
        assert_eq!(
            output.pointer("/answerPolicy/no_default_product"),
            Some(&json!(true))
        );
        assert!(output.pointer("/risks/0/metric").is_some());
    }

    #[tokio::test]
    async fn query_product_metrics_defers_without_explicit_metrics() {
        let outcome = execute(
            "query_product_metrics",
            json!({"product_name": "冷萃咖啡", "days": 30}),
            &SkillContext::default(),
        )
        .await
        .expect("query_product_metrics should return structured deferred");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("product_metrics_required")
        );
        assert!(
            output
                .get("deferred_reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("不会默认读取任何预设商品")
        );
        assert_eq!(output.pointer("/query/product_ref"), Some(&Value::Null));
    }

    #[tokio::test]
    async fn query_campaign_metrics_marks_success_for_explicit_metrics() {
        let outcome = execute(
            "query_campaign_metrics",
            json!({
                "campaign_id": "camp-618",
                "campaign_name": "618大促",
                "campaign_type": "promotion",
                "platform": "天猫",
                "budget": 50000.0,
                "spend": 42000.0,
                "gmv": 210000.0,
                "previous_gmv": 160000.0,
                "orders": 1800,
                "visitors": 36000,
                "clicks": 7200,
                "impressions": 180000,
                "coupon_cost": 12000.0,
                "refund_amount": 6000.0
            }),
            &SkillContext::default(),
        )
        .await
        .expect("query_campaign_metrics should run from explicit metrics");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("explicit_campaign_metrics")
        );
        assert_eq!(output.pointer("/totals/gmv"), Some(&json!(210000.0)));
        assert_eq!(output.pointer("/totals/total_cost"), Some(&json!(54000.0)));
        assert_eq!(output.pointer("/derived_metrics/roas"), Some(&json!(5.0)));
    }

    #[tokio::test]
    async fn query_campaign_metrics_defers_without_explicit_metrics() {
        let outcome = execute(
            "query_campaign_metrics",
            json!({"campaign_name": "五一活动", "days": 30}),
            &SkillContext::default(),
        )
        .await
        .expect("query_campaign_metrics should return structured deferred");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("campaign_metrics_required")
        );
        assert!(
            output
                .get("deferred_reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("不会伪造活动")
        );
    }

    #[test]
    fn data_dashboard_builds_from_user_metrics() {
        let output = data_dashboard(&json!({
            "days": 14,
            "platform": "douyin",
            "gmv": 180000.0,
            "previous_gmv": 150000.0,
            "orders": 1800,
            "previous_orders": 1500,
            "visitors": 36000,
            "previous_visitors": 33000,
            "clicks": 7200,
            "ad_spend": 36000.0,
            "refund_amount": 5400.0,
            "gross_profit": 54000.0,
            "target_gmv": 200000.0
        }));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert!(output.get("核心指标").and_then(Value::as_array).is_some());
        assert!(output.pointer("/经营健康度/总分").is_some());
    }

    #[test]
    fn data_dashboard_marks_sample_when_metrics_missing() {
        let output = data_dashboard(&json!({"days": 30}));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(false));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("store_metrics_required")
        );
        assert!(
            output
                .get("说明")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("不能当作平台实时数据")
        );
    }

    #[test]
    fn data_funnel_analysis_marks_success_for_user_data() {
        let output = funnel_analysis(&json!({
            "impressions": 10000,
            "clicks": 620,
            "add_to_cart": 180,
            "orders": 82,
            "payments": 78
        }));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert!(
            output
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("漏斗分析完成")
        );
    }

    #[test]
    fn data_funnel_analysis_defers_when_metrics_missing() {
        let output = funnel_analysis(&json!({}));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("skill").and_then(Value::as_str),
            Some("data_funnel_analysis")
        );
    }

    #[test]
    fn data_channel_roi_marks_success_for_user_channels() {
        let output = channel_roi(&json!({
            "channels": [
                {"name": "douyin", "gmv": 120000.0, "ad_spend": 30000.0, "orders": 600, "uv": 20000},
                {"name": "taobao", "gmv": 80000.0, "ad_spend": 26000.0, "orders": 420, "uv": 16000}
            ]
        }));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert!(
            output
                .get("渠道ROI排名")
                .and_then(Value::as_array)
                .is_some()
        );
    }

    #[tokio::test]
    async fn data_attribution_analysis_marks_success_for_user_channels() {
        let outcome = execute(
            "data_attribution_analysis",
            json!({
                "days": 30,
                "channels": [
                    {"name": "抖音", "gmv": 180000.0, "uv": 50000.0, "conversion_rate": 3.2, "ad_spend": 48000.0},
                    {"name": "淘宝", "gmv": 120000.0, "uv": 42000.0, "conversion_rate": 2.4, "ad_spend": 22000.0},
                    {"name": "小红书", "gmv": 70000.0, "uv": 38000.0, "conversion_rate": 1.6, "ad_spend": 18000.0}
                ]
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data attribution analysis should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str).map(|value| {
                value.contains("渠道归因分析完成") && value.contains("归因贡献最高")
            }),
            Some(true)
        );
        assert!(
            output
                .get("各渠道归因明细")
                .and_then(Value::as_array)
                .is_some()
        );
    }

    #[tokio::test]
    async fn data_attribution_analysis_defers_without_channels() {
        let outcome = execute(
            "data_attribution_analysis",
            json!({}),
            &SkillContext::default(),
        )
        .await
        .expect("data attribution analysis should return deferred output");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("skill").and_then(Value::as_str),
            Some("data_attribution_analysis")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("渠道归因分析报告已生成")
        );
    }

    #[tokio::test]
    async fn data_comprehensive_diagnosis_marks_success_for_user_inputs() {
        let outcome = execute(
            "data_comprehensive_diagnosis",
            json!({
                "days": 30,
                "focus": "growth",
                "gmv": 260000.0,
                "previous_gmv": 230000.0,
                "orders": 2100,
                "previous_orders": 1800,
                "visitors": 62000,
                "previous_visitors": 58000,
                "clicks": 8600,
                "ad_spend": 72000.0,
                "refund_amount": 15600.0,
                "gross_profit": 68000.0,
                "target_gmv": 300000.0,
                "impressions": 180000,
                "add_to_cart": 6200,
                "payments": 1990,
                "values": [7200.0, 7350.0, 7480.0, 7600.0, 9100.0, 7800.0, 8050.0],
                "channels": [
                    {"name": "抖音", "gmv": 140000.0, "uv": 32000.0, "conversion_rate": 3.1, "ad_spend": 42000.0},
                    {"name": "淘宝", "gmv": 90000.0, "uv": 25000.0, "conversion_rate": 2.6, "ad_spend": 21000.0}
                ],
                "items": [
                    {"name": "口味不符", "refund_amount": 9200.0},
                    {"name": "包装破损", "refund_amount": 4100.0}
                ],
                "total_customers": 3400,
                "avg_order_value": 124.0,
                "avg_orders_per_customer": 1.9,
                "recency_days": 28.0,
                "cac": 38.0,
                "purchase_frequency": 1.7,
                "gross_margin": 31.0,
                "monthly_retention": 42.0
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data comprehensive diagnosis should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("summary").and_then(Value::as_str).map(|value| {
                value.contains("综合业务诊断完成") && value.contains("健康评分")
            }),
            Some(true)
        );
        assert!(
            output
                .get("分项诊断")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() >= 6)
        );
        assert!(
            output
                .get("综合诊断报告")
                .and_then(Value::as_str)
                .is_some_and(|value| value.contains("7天行动计划"))
        );
        assert!(
            output
                .get("7天行动计划")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())
        );
    }

    #[tokio::test]
    async fn data_comprehensive_diagnosis_defers_without_useful_inputs() {
        let outcome = execute(
            "data_comprehensive_diagnosis",
            json!({"days": 30, "focus": "risk"}),
            &SkillContext::default(),
        )
        .await
        .expect("data comprehensive diagnosis should return deferred output");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("skill").and_then(Value::as_str),
            Some("data_comprehensive_diagnosis")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("user_input_required")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("综合业务诊断报告已生成")
        );
    }

    #[test]
    fn data_refund_decomposition_marks_success_for_user_items() {
        let output = refund_decomposition(&json!({
            "items": [
                {"name": "尺码不符", "refund_amount": 24000.0},
                {"name": "物流破损", "refund_amount": 9000.0},
                {"name": "色差", "gmv": 100000.0, "refund_rate": 0.04}
            ]
        }));

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert!(output.get("帕累托分析").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn data_anomaly_diagnosis_marks_success_for_user_series() {
        let outcome = execute(
            "data_anomaly_diagnosis",
            json!({
                "metric_name": "GMV",
                "values": [12000.0, 12100.0, 11950.0, 12200.0, 11880.0, 18800.0],
                "threshold": 1.5
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data anomaly diagnosis should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("异常诊断报告已生成")
        );
        assert!(output.get("异常详情").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn data_anomaly_diagnosis_defers_without_enough_series() {
        let outcome = execute(
            "data_anomaly_diagnosis",
            json!({"values": [12000.0, 12100.0]}),
            &SkillContext::default(),
        )
        .await
        .expect("data anomaly diagnosis should return deferred output");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("user_input_required")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("missing_user_input")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("异常诊断报告已生成")
        );
    }

    #[tokio::test]
    async fn data_trend_forecast_marks_success_for_user_series() {
        let outcome = execute(
            "data_trend_forecast",
            json!({
                "metric_name": "gmv",
                "values": [100.0, 112.0, 124.0, 141.0],
                "forecast_periods": 3
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data trend forecast should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("趋势预测报告已生成")
        );
        assert!(output.get("预测结果").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn data_customer_segmentation_discloses_default_inputs() {
        let outcome = execute(
            "data_customer_segmentation",
            json!({}),
            &SkillContext::default(),
        )
        .await
        .expect("data customer segmentation should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("客户分层报告已生成")
        );
        assert!(output.get("分群").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn data_competitor_analysis_discloses_default_competitors() {
        let outcome = execute(
            "data_competitor_analysis",
            json!({
                "our_product": "冷萃咖啡",
                "our_price": 99.0
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data competitor analysis should run with deterministic competitors");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("竞品分析报告已生成")
        );
        assert!(output.get("竞品矩阵").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn data_multi_period_trend_marks_success_for_user_series() {
        let outcome = execute(
            "data_multi_period_trend",
            json!({
                "metric": "gmv",
                "periods": [2, 4],
                "values": [100.0, 108.0, 115.0, 126.0, 140.0, 152.0, 161.0, 180.0]
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data multi-period trend should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert!(output.get("summary").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn data_ltv_calculator_discloses_default_inputs() {
        let outcome = execute("data_ltv_calculator", json!({}), &SkillContext::default())
            .await
            .expect("data LTV calculator should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert!(output.get("summary").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn data_cohort_analysis_discloses_default_inputs() {
        let outcome = execute("data_cohort_analysis", json!({}), &SkillContext::default())
            .await
            .expect("data cohort analysis should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert!(output.get("summary").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn data_seasonal_decompose_defers_without_enough_series() {
        let outcome = execute(
            "data_seasonal_decompose",
            json!({
                "period": 7,
                "values": [100.0, 108.0, 115.0]
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data seasonal decomposition should return deferred output");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("user_input_required")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("missing_user_input")
        );
    }

    #[tokio::test]
    async fn data_price_elasticity_discloses_default_points() {
        let outcome = execute("data_price_elasticity", json!({}), &SkillContext::default())
            .await
            .expect("data price elasticity should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert!(output.get("summary").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn data_demand_forecast_marks_success_for_user_series() {
        let outcome = execute(
            "data_demand_forecast",
            json!({
                "metric": "orders",
                "forecast_days": [7, 14],
                "values": [120.0, 128.0, 134.0, 145.0, 151.0, 162.0]
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data demand forecast should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert!(output.get("summary").and_then(Value::as_str).is_some());
    }

    #[tokio::test]
    async fn data_ab_test_analyzer_marks_success_for_experiment() {
        let outcome = execute(
            "data_ab_test_analyzer",
            json!({
                "test_name": "详情页首屏改版",
                "control_visitors": 10000,
                "control_conversions": 320,
                "test_visitors": 9800,
                "test_conversions": 390
            }),
            &SkillContext::default(),
        )
        .await
        .expect("data A/B test analyzer should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("A/B测试分析报告已生成")
        );
        assert!(output.get("置信度").and_then(Value::as_str).is_some());
    }
}
