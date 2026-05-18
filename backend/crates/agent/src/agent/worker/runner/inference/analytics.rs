fn infer_demand_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_price_quantity_points(text: &str) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_price_quantity_segment)
        .collect()
}

fn parse_price_quantity_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return None;
    }
    let price = extract_last_metric_value(trimmed, &["价格点", "售价", "单价", "price", "价格"])?;
    let quantity = extract_metric_value(
        trimmed,
        &["销量", "销售量", "件数", "销量件数", "quantity", "qty"],
    )?;
    Some(serde_json::json!({
        "price": price,
        "quantity": quantity,
    }))
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
        if let Some(value) = parse_metric_number_after_alias(&text[start ..]) {
            return Some(value);
        }
    }
    None
}

fn store_metrics_input_has_metrics(input: &serde_json::Map<String, serde_json::Value>) -> bool {
    [
        "gmv",
        "previous_gmv",
        "orders",
        "previous_orders",
        "visitors",
        "previous_visitors",
        "clicks",
        "ad_spend",
        "refund_amount",
        "gross_profit",
    ]
    .iter()
    .any(|key| input.contains_key(*key))
}

fn metric_query_input_has_metrics(input: &serde_json::Map<String, serde_json::Value>) -> bool {
    [
        "gmv",
        "previous_gmv",
        "orders",
        "visitors",
        "clicks",
        "ad_spend",
        "refund_amount",
        "gross_profit",
        "inventory",
        "stockout_count",
        "budget",
        "spend",
        "impressions",
        "coupon_cost",
    ]
    .iter()
    .any(|key| input.contains_key(*key))
}

fn infer_channel_roi_rows(text: &str) -> Vec<serde_json::Value> {
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
        if let Some(value) = gmv {
            row.insert("gmv".to_string(), serde_json::json!(value));
        }
        if let Some(value) = ad_spend {
            row.insert("ad_spend".to_string(), serde_json::json!(value));
        }
        if let Some(value) = orders {
            row.insert(
                "orders".to_string(),
                serde_json::json!(value.round() as i64),
            );
        }
        if let Some(value) = uv {
            row.insert("uv".to_string(), serde_json::json!(value.round() as i64));
        }
        if let Some(value) = conversion_rate {
            row.insert("conversion_rate".to_string(), serde_json::json!(value));
        }
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
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

fn infer_attribution_model(text: &str) -> &'static str {
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

fn infer_refund_items(text: &str) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_refund_item_segment)
        .collect()
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

