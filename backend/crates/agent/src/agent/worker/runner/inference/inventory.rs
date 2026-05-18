use super::{
    metrics::extract_metric_value,
    numbers::{infer_explicit_period_days, numbers_in_text},
};

pub(in crate::agent::worker::runner) fn infer_avg_daily_demand(text: &str) -> f64 {
    extract_metric_value(
        text,
        &["日均销量", "日均需求", "日销量", "avg_daily_demand"],
    )
    .unwrap_or(1.0)
    .max(0.0)
}

pub(in crate::agent::worker::runner) fn infer_lead_time_days(text: &str, default: i64) -> i64 {
    extract_metric_value(
        text,
        &["供货周期", "补货周期", "交货周期", "lead_time_days"],
    )
    .map(|value| value.round().max(1.0) as i64)
    .or_else(|| infer_explicit_period_days(text))
    .unwrap_or(default.max(1))
}

pub(in crate::agent::worker::runner) fn infer_ordering_cost(text: &str) -> Option<f64> {
    extract_metric_value(text, &["订货成本", "下单成本", "ordering_cost"])
}

pub(in crate::agent::worker::runner) fn infer_holding_rate(text: &str) -> Option<f64> {
    extract_metric_value(text, &["持有成本率", "库存持有成本率", "holding_rate"])
}

pub(in crate::agent::worker::runner) fn infer_total_budget(text: &str) -> Option<f64> {
    extract_metric_value(text, &["总预算", "广告总预算", "total_budget"])
}

pub(in crate::agent::worker::runner) fn infer_service_level(text: &str) -> Option<String> {
    for level in ["99%", "98%", "95%", "90%"] {
        if text.contains(level) {
            return Some(level.to_string());
        }
    }
    let lower = text.to_ascii_lowercase();
    for (needle, level) in [
        ("service_level99", "99%"),
        ("service_level98", "98%"),
        ("service_level95", "95%"),
        ("service_level90", "90%"),
        ("service level 99", "99%"),
        ("service level 98", "98%"),
        ("service level 95", "95%"),
        ("service level 90", "90%"),
    ] {
        if lower.contains(needle) {
            return Some(level.to_string());
        }
    }
    None
}

pub(in crate::agent::worker::runner) fn infer_inventory_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_inventory_sku_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_ad_fatigue_channels(
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
        "直通车",
        "千川",
    ];

    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in PLATFORM_LABELS {
        let Some(segment) = segment_after_label(text, label, PLATFORM_LABELS) else {
            continue;
        };
        let name = canonical_ad_channel_name(label);
        if seen.iter().any(|value| value == name) {
            continue;
        }
        let ctr_baseline = extract_metric_value(
            &segment,
            &[
                "CTR基准",
                "基准CTR",
                "历史CTR",
                "baseline_ctr",
                "ctr_baseline",
            ],
        );
        let ctr_recent = extract_metric_value(
            &segment,
            &["近期CTR", "当前CTR", "最近CTR", "recent_ctr", "ctr_recent"],
        );
        let impressions_per_user = extract_metric_value(
            &segment,
            &[
                "曝光频次",
                "频次",
                "人均曝光",
                "impressions_per_user",
                "frequency",
            ],
        );
        let budget = extract_metric_value(
            &segment,
            &["预算", "广告预算", "投放预算", "budget", "ad_budget"],
        );
        let roas = extract_metric_value(&segment, &["ROAS", "roas", "投产", "roi", "ROI"]);

        if [ctr_baseline, ctr_recent, impressions_per_user, budget, roas]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        let mut row = serde_json::Map::new();
        row.insert("name".to_string(), serde_json::json!(name));
        insert_optional_number(&mut row, "ctr_baseline", ctr_baseline);
        insert_optional_number(&mut row, "ctr_recent", ctr_recent);
        insert_optional_number(&mut row, "impressions_per_user", impressions_per_user);
        insert_optional_number(&mut row, "budget", budget);
        insert_optional_number(&mut row, "roas", roas);
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

pub(in crate::agent::worker::runner) fn infer_abc_xyz_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_abc_xyz_sku_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_top_n(text: &str) -> Option<i64> {
    extract_metric_value(text, &["top_n", "前N", "前n", "展示前"])
        .map(|value| value.round().clamp(1.0, 100.0) as i64)
}

fn parse_inventory_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(trimmed, &["sku", "SKU", "款", "货品", "商品"]) {
        return None;
    }
    let monthly_sales = extract_last_metric_value(
        trimmed,
        &["月销量", "月销", "monthly_sales", "monthly sales", "销量"],
    )?;
    let cost = extract_last_metric_value(
        trimmed,
        &["单位成本", "单品成本", "成本", "cost", "unit_cost"],
    )?;
    let stock_qty = extract_last_metric_value(
        trimmed,
        &[
            "当前库存",
            "库存数量",
            "库存",
            "stock_qty",
            "stock",
            "inventory",
        ],
    )?;
    Some(serde_json::json!({
        "name": infer_inventory_sku_name(trimmed),
        "monthly_sales": monthly_sales,
        "cost": cost,
        "stock_qty": stock_qty,
    }))
}

fn parse_abc_xyz_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(trimmed, &["sku", "SKU", "款", "货品", "商品"]) {
        return None;
    }
    let gmv = extract_last_metric_value(trimmed, &["GMV", "gmv", "销售额", "成交额", "营收"])?;
    let history = infer_abc_xyz_sales_history(trimmed);
    let mut row = serde_json::Map::new();
    row.insert(
        "sku_id".to_string(),
        serde_json::json!(infer_abc_xyz_sku_id(trimmed)),
    );
    row.insert("gmv".to_string(), serde_json::json!(gmv));
    if !history.is_empty() {
        row.insert("sales_history".to_string(), serde_json::json!(history));
    }
    Some(serde_json::Value::Object(row))
}

fn infer_inventory_sku_name(segment: &str) -> String {
    let trimmed = normalize_sku_prefix(segment);
    let metric_markers = [
        "月销量",
        "月销",
        "monthly_sales",
        "monthly sales",
        "销量",
        "单位成本",
        "单品成本",
        "成本",
        "cost",
        "unit_cost",
        "当前库存",
        "库存数量",
        "库存",
        "stock_qty",
        "stock",
        "inventory",
    ];
    clean_sku_label(trimmed, &metric_markers, "SKU")
}

fn infer_abc_xyz_sku_id(segment: &str) -> String {
    let trimmed = normalize_sku_prefix(segment);
    let metric_markers = [
        "GMV",
        "gmv",
        "销售额",
        "成交额",
        "营收",
        "历史销量",
        "销量历史",
        "sales_history",
    ];
    clean_sku_label(trimmed, &metric_markers, "SKU")
}

fn normalize_sku_prefix(segment: &str) -> &str {
    let trimmed = segment
        .trim()
        .trim_start_matches(['-', '•', '*', ' '])
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    lower
        .rfind("sku")
        .map(|index| trimmed[index ..].trim())
        .unwrap_or(trimmed)
}

fn clean_sku_label(segment: &str, markers: &[&str], fallback: &str) -> String {
    let lower = segment.to_ascii_lowercase();
    let end = markers
        .iter()
        .filter_map(|marker| lower.find(&marker.to_ascii_lowercase()))
        .min()
        .unwrap_or(segment.len());
    let raw = segment[.. end]
        .trim()
        .trim_matches(['：', ':', ',', '，', ';', '；', ' '])
        .trim();
    let raw = raw
        .rfind(['：', ':'])
        .map(|index| raw[index + 1 ..].trim())
        .unwrap_or(raw);
    let lower_raw = raw.to_ascii_lowercase();
    let raw = lower_raw
        .rfind("sku")
        .map(|index| raw[index ..].trim())
        .unwrap_or(raw);
    if raw.is_empty() {
        fallback.to_string()
    } else {
        raw.to_string()
    }
}

fn infer_abc_xyz_sales_history(segment: &str) -> Vec<f64> {
    let Some(history_segment) =
        segment_after_abc_history_label(segment, &["历史销量", "销量历史", "sales_history"])
    else {
        return Vec::new();
    };
    numbers_in_text(&history_segment)
}

fn segment_after_abc_history_label(text: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        if let Some(index) = lower.find(&label_lower)
            && best
                .map(|(best_index, _)| index < best_index)
                .unwrap_or(true)
        {
            best = Some((index, *label));
        }
    }
    let (index, label) = best?;
    let start = index + label.len();
    let tail = text[start ..].trim();
    let end = tail.find(['；', ';', '\n']).unwrap_or(tail.len());
    Some(tail[.. end].trim().to_string())
}

fn extract_last_metric_value(text: &str, aliases: &[&str]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias_lower = alias.to_ascii_lowercase();
        let index = lower
            .rmatch_indices(&alias_lower)
            .next()
            .map(|(index, _)| index);
        let Some(index) = index else {
            continue;
        };
        let start = index + alias_lower.len();
        if let Some(value) = super::metrics::parse_metric_number_after_alias(&text[start ..]) {
            return Some(value);
        }
    }
    None
}

fn segment_after_label(text: &str, label: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();
    let index = lower.find(&label_lower)?;
    let start = index + label.len();
    let tail = &text[start ..];
    let tail_lower = tail.to_ascii_lowercase();
    let next = labels
        .iter()
        .filter(|other| other.to_ascii_lowercase() != label_lower)
        .filter_map(|other| tail_lower.find(&other.to_ascii_lowercase()))
        .min()
        .unwrap_or(tail.len());
    Some(tail[.. next].to_string())
}

fn canonical_ad_channel_name(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if matches!(label, "直通车") {
        "直通车"
    } else if matches!(label, "千川") {
        "千川"
    } else {
        canonical_channel_name(lower.as_str())
    }
}

fn canonical_channel_name(label: &str) -> &'static str {
    match label {
        "douyin" | "tiktok" | "抖音" => "抖音",
        "taobao" | "淘宝" => "淘宝",
        "tmall" | "天猫" => "天猫",
        "jd" | "京东" => "京东",
        "pdd" | "拼多多" => "拼多多",
        "xiaohongshu" | "小红书" => "小红书",
        "kuaishou" | "快手" => "快手",
        _ => "其他",
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

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
