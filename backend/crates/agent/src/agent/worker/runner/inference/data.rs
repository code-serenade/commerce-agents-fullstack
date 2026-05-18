use super::{
    metrics::extract_metric_value,
    numbers::{infer_period_days, numbers_in_text},
};

pub(in crate::agent::worker::runner) fn infer_values(text: &str) -> Vec<f64> {
    numbers_in_text(text)
}

pub(in crate::agent::worker::runner) fn infer_channel_roi_rows(
    text: &str,
) -> Vec<serde_json::Value> {
    const PLATFORM_LABELS: &[&str] = &[
        "抖音",
        "douyin",
        "tiktok",
        "淘宝",
        "taobao",
        "天猫",
        "tmall",
        "京东",
        "jd",
        "拼多多",
        "pdd",
        "小红书",
        "xiaohongshu",
        "快手",
        "kuaishou",
    ];

    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in PLATFORM_LABELS {
        let Some(segment) = segment_after_label(text, label, PLATFORM_LABELS) else {
            continue;
        };
        let name = canonical_channel_name(label);
        if seen.iter().any(|value| value == name) {
            continue;
        }

        let gmv = extract_metric_value(&segment, &["GMV", "gmv", "销售额", "成交额", "营收"]);
        let ad_spend = extract_metric_value(
            &segment,
            &[
                "广告花费",
                "广告费",
                "投放花费",
                "投放消耗",
                "消耗",
                "花费",
                "ad_spend",
                "ad cost",
            ],
        );
        let orders = extract_metric_value(&segment, &["订单数", "订单", "支付订单", "orders"]);
        let uv = extract_metric_value(&segment, &["访客数", "访客", "UV", "uv"]);
        let conversion_rate = extract_metric_value(
            &segment,
            &["转化率", "成交转化率", "conversion_rate", "cvr"],
        );

        if [gmv, ad_spend, orders, uv, conversion_rate]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        let mut row = serde_json::Map::new();
        row.insert("name".to_string(), serde_json::json!(name));
        insert_optional_number(&mut row, "gmv", gmv);
        insert_optional_number(&mut row, "ad_spend", ad_spend);
        insert_optional_integer(&mut row, "orders", orders);
        insert_optional_integer(&mut row, "uv", uv);
        insert_optional_number(&mut row, "conversion_rate", conversion_rate);
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

pub(in crate::agent::worker::runner) fn infer_attribution_model(text: &str) -> &'static str {
    if contains_any(text, &["last_touch", "末次", "最后触点", "末次归因"]) {
        "last_touch"
    } else if contains_any(text, &["linear", "线性", "线性归因"]) {
        "linear"
    } else if contains_any(text, &["time_decay", "时间衰减", "衰减归因"]) {
        "time_decay"
    } else if contains_any(text, &["position", "位置归因", "u形", "U形", "首末"]) {
        "position"
    } else {
        "all"
    }
}

pub(in crate::agent::worker::runner) fn infer_refund_items(text: &str) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_refund_item_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_metric_number(
    text: &str,
    metric: &str,
) -> Option<f64> {
    match metric {
        "cac" => extract_metric_value(text, &["CAC", "cac", "获客成本", "获客成本CAC"]),
        "avg_order_value" => extract_metric_value(
            text,
            &["客单价", "平均客单价", "AOV", "aov", "avg_order_value"],
        ),
        "purchase_frequency" => extract_metric_value(
            text,
            &["购买频次", "月购买频次", "复购频次", "purchase_frequency"],
        ),
        "gross_margin" => extract_metric_value(text, &["毛利率", "gross_margin"]),
        "monthly_retention" => {
            extract_metric_value(text, &["月留存率", "留存率", "monthly_retention"])
        }
        "new_customers" => extract_metric_value(
            text,
            &["首期新客", "新客数", "新增客户", "新客", "new_customers"],
        ),
        "total_customers" => extract_metric_value(
            text,
            &[
                "客户总数",
                "总客户数",
                "客户数",
                "用户数",
                "total_customers",
            ],
        ),
        "avg_orders_per_customer" => extract_metric_value(
            text,
            &[
                "人均订单数",
                "人均购买次数",
                "平均订单数",
                "avg_orders_per_customer",
            ],
        ),
        "recency_days" => extract_metric_value(
            text,
            &["平均购买间隔", "购买间隔", "最近购买间隔", "recency_days"],
        ),
        "forecast_periods" => extract_metric_value(text, &["未来", "预测", "forecast_periods"])
            .map(|value| value.round().clamp(1.0, 90.0)),
        "target_gmv" => extract_metric_value(
            text,
            &[
                "目标GMV",
                "目标销售额",
                "目标成交额",
                "target_gmv",
                "goal_gmv",
            ],
        ),
        "previous_gmv" => extract_metric_value(
            text,
            &["previous_gmv", "last_gmv", "上期GMV", "上周GMV", "前期GMV"],
        ),
        "gmv" => extract_metric_value(
            text,
            &[
                "本期GMV",
                "当前GMV",
                "活动GMV",
                "商品GMV",
                "GMV",
                "gmv",
                "销售额",
                "成交额",
                "营收",
            ],
        ),
        "previous_orders" => extract_metric_value(
            text,
            &[
                "previous_orders",
                "last_orders",
                "上期订单",
                "上周订单",
                "前期订单",
            ],
        ),
        "orders" => extract_metric_value(
            text,
            &[
                "本期订单",
                "当前订单",
                "订单数",
                "订单",
                "成交订单",
                "支付订单",
            ],
        ),
        "previous_visitors" => extract_metric_value(
            text,
            &[
                "previous_visitors",
                "last_visitors",
                "上期UV",
                "上周UV",
                "前期UV",
            ],
        ),
        "visitors" => {
            extract_metric_value(text, &["本期UV", "当前UV", "UV", "uv", "访客数", "访客"])
        }
        "clicks" => extract_metric_value(text, &["点击数", "点击", "clicks", "click_count"]),
        "ad_spend" => extract_metric_value(
            text,
            &[
                "广告费",
                "广告花费",
                "投放费用",
                "ad_spend",
                "ad spend",
                "ad_cost",
            ],
        ),
        "refund_amount" => extract_metric_value(
            text,
            &["退款金额", "退款额", "退款", "refund_amount", "refunds"],
        ),
        "gross_profit" => extract_metric_value(text, &["毛利额", "毛利", "gross_profit", "profit"]),
        "impressions" => extract_metric_value(
            text,
            &[
                "曝光",
                "曝光量",
                "展现",
                "展现量",
                "impressions",
                "exposure",
            ],
        ),
        "add_to_cart" => extract_metric_value(text, &["加购", "加购数", "add_to_cart"]),
        "payments" => extract_metric_value(text, &["付款数", "付款", "支付数", "支付", "payments"]),
        "inventory" => extract_metric_value(text, &["库存", "剩余库存", "inventory", "stock"]),
        "stockout_count" => extract_metric_value(
            text,
            &["缺货次数", "断货次数", "stockout_count", "stockout"],
        ),
        "budget" => extract_metric_value(text, &["预算", "budget", "活动预算"]),
        "spend" => extract_metric_value(
            text,
            &["活动花费", "实际花费", "已花费", "消耗", "spend", "cost"],
        ),
        "coupon_cost" => {
            extract_metric_value(text, &["优惠券成本", "券成本", "让利成本", "coupon_cost"])
        }
        _ => None,
    }
}

pub(in crate::agent::worker::runner) fn infer_metric_integer(
    text: &str,
    metric: &str,
) -> Option<i64> {
    infer_metric_number(text, metric).map(|value| value.round().max(0.0) as i64)
}

pub(in crate::agent::worker::runner) fn infer_ab_integer(text: &str, key: &str) -> Option<i64> {
    let aliases: &[&str] = match key {
        "control_visitors" => &[
            "control_visitors",
            "control visitors",
            "对照组访客数",
            "对照组访客",
            "控制组访客数",
            "控制组访客",
            "A组访客数",
            "A组访客",
            "A版访客",
        ],
        "control_conversions" => &[
            "control_conversions",
            "control conversions",
            "对照组转化数",
            "对照组转化",
            "控制组转化数",
            "控制组转化",
            "A组转化数",
            "A组转化",
            "A版转化",
        ],
        "test_visitors" => &[
            "test_visitors",
            "test visitors",
            "实验组访客数",
            "实验组访客",
            "测试组访客数",
            "测试组访客",
            "B组访客数",
            "B组访客",
            "B版访客",
        ],
        "test_conversions" => &[
            "test_conversions",
            "test conversions",
            "实验组转化数",
            "实验组转化",
            "测试组转化数",
            "测试组转化",
            "B组转化数",
            "B组转化",
            "B版转化",
        ],
        _ => &[],
    };
    extract_metric_value(text, aliases)
        .map(|value| value.round().max(0.0) as i64)
        .or_else(|| infer_ab_number_fallback(text, key))
}

fn infer_ab_number_fallback(text: &str, key: &str) -> Option<i64> {
    let numbers = numbers_in_text(text);
    if numbers.len() < 4 {
        return None;
    }
    let index = match key {
        "control_visitors" => 0,
        "control_conversions" => 1,
        "test_visitors" => 2,
        "test_conversions" => 3,
        _ => return None,
    };
    numbers
        .get(index)
        .map(|value| value.round().max(0.0) as i64)
}

pub(in crate::agent::worker::runner) fn infer_ab_test_name(text: &str) -> &'static str {
    if contains_any(text, &["首屏", "详情首屏", "详情页首屏", "above the fold"]) {
        "详情页首屏A/B测试"
    } else if contains_any(text, &["主图", "封面", "image"]) {
        "主图A/B测试"
    } else if contains_any(text, &["标题", "title"]) {
        "标题A/B测试"
    } else if contains_any(text, &["详情页", "详情", "detail"]) {
        "详情页A/B测试"
    } else if contains_any(text, &["直播", "live"]) {
        "直播间A/B测试"
    } else {
        "A/B测试"
    }
}

pub(in crate::agent::worker::runner) fn infer_ab_metric(text: &str) -> &'static str {
    if contains_any(text, &["点击", "ctr", "CTR", "click"]) {
        "点击率"
    } else if contains_any(text, &["支付", "成交", "订单", "purchase", "order"]) {
        "支付转化率"
    } else {
        "转化率"
    }
}

pub(in crate::agent::worker::runner) fn infer_multi_periods(text: &str) -> Vec<i64> {
    if contains_any(text, &["周内", "周度", "每周", "weekly", "week"]) {
        vec![7, 14, 28]
    } else {
        vec![7, 14, 30]
    }
}

pub(in crate::agent::worker::runner) fn infer_forecast_periods(text: &str) -> i64 {
    infer_metric_number(text, "forecast_periods")
        .map(|value| value.round().clamp(1.0, 90.0) as i64)
        .unwrap_or(7)
}

pub(in crate::agent::worker::runner) fn infer_top_n(text: &str, default: i64) -> i64 {
    infer_metric_number(text, "top_n")
        .map(|value| value.round().clamp(1.0, 100.0) as i64)
        .unwrap_or(default)
}

pub(in crate::agent::worker::runner) fn infer_diagnosis_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "重点看增长",
            "重点分析增长",
            "聚焦增长",
            "增长为主",
            "增长优先",
            "focus growth",
        ],
    ) {
        "growth"
    } else if contains_any(text, &["profit", "利润", "毛利", "盈利", "ROI", "roi"]) {
        "profit"
    } else if contains_any(text, &["retention", "留存", "复购", "客户", "会员"]) {
        "retention"
    } else if contains_any(text, &["risk", "风险", "异常", "退款", "下滑"]) {
        "risk"
    } else if contains_any(text, &["growth", "增长", "放量", "规模", "GMV", "gmv"]) {
        "growth"
    } else {
        "all"
    }
}

pub(in crate::agent::worker::runner) fn infer_labeled_token(
    text: &str,
    token: &str,
) -> Option<String> {
    let labels: &[&str] = match token {
        "product_id" => &["商品ID", "product_id", "product id"],
        "sku_id" => &["SKU", "sku_id", "sku id", "规格ID"],
        "campaign_id" => &["活动ID", "campaign_id", "campaign id"],
        _ => &[],
    };
    super::domain::extract_labeled_token(text, labels)
}

pub(in crate::agent::worker::runner) fn default_days(text: &str) -> i64 {
    infer_period_days(text)
}

fn parse_refund_item_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.rsplit(['：', ':']).next().unwrap_or(segment).trim();
    if trimmed.is_empty() {
        return None;
    }
    let amount =
        extract_metric_value(trimmed, &["退款金额", "退款额", "退款", "退货金额", "退货"])?;
    let name = refund_item_name(trimmed).unwrap_or("综合退款");
    Some(serde_json::json!({
        "name": name,
        "refund_amount": amount,
    }))
}

fn refund_item_name(segment: &str) -> Option<&str> {
    for alias in ["退款金额", "退款额", "退款", "退货金额", "退货"] {
        if let Some(index) = segment.find(alias) {
            let name = segment[.. index]
                .trim()
                .trim_matches(['：', ':', '-', '，', ',', '；', ';', ' ']);
            return if name.is_empty() { None } else { Some(name) };
        }
    }
    None
}

fn segment_after_label(text: &str, label: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();
    let index = lower.find(&label_lower)?;
    let start = index + label.len();
    let mut end = text.len();

    for other in labels {
        if *other == label {
            continue;
        }
        let other_lower = other.to_ascii_lowercase();
        if let Some(offset) = lower[start ..].find(&other_lower) {
            let candidate = start + offset;
            if candidate > start && candidate < end {
                end = candidate;
            }
        }
    }

    Some(text[start .. end].trim().to_string())
}

fn canonical_channel_name(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if matches!(label, "抖音") || matches!(lower.as_str(), "douyin" | "tiktok") {
        "抖音"
    } else if matches!(label, "天猫") || lower == "tmall" {
        "天猫"
    } else if matches!(label, "淘宝") || lower == "taobao" {
        "淘宝"
    } else if matches!(label, "京东") || lower == "jd" {
        "京东"
    } else if matches!(label, "拼多多") || lower == "pdd" {
        "拼多多"
    } else if matches!(label, "小红书") || lower == "xiaohongshu" {
        "小红书"
    } else if matches!(label, "快手") || lower == "kuaishou" {
        "快手"
    } else {
        "未知渠道"
    }
}

fn insert_optional_number(
    row: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<f64>,
) {
    if let Some(value) = value {
        row.insert(key.to_string(), serde_json::json!(value));
    }
}

fn insert_optional_integer(
    row: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<f64>,
) {
    if let Some(value) = value {
        row.insert(
            key.to_string(),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
