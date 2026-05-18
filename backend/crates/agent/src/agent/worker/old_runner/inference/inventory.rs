fn infer_inventory_planning_input(
    text: &str,
    task: &Task,
    avg_daily_demand: f64,
    lead_time_days: i64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "avg_daily_demand".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["日均销量", "日均需求", "日销量", "avg_daily_demand"]
            )
            .unwrap_or(avg_daily_demand)
            .max(0.0)
        ),
    );
    input.insert(
        "lead_time_days".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["供货周期", "补货周期", "交货周期", "lead_time_days"]
            )
            .map(|value| value.round().max(1.0) as i64)
            .unwrap_or(lead_time_days.max(1))
        ),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}


fn infer_inventory_optimizer_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let skus = infer_inventory_skus(text);
    if !skus.is_empty() {
        input.insert("skus".to_string(), serde_json::json!(skus));
    }
    insert_metric_value(
        &mut input,
        "ordering_cost",
        text,
        &["订货成本", "下单成本", "ordering_cost"],
    );
    insert_metric_value(
        &mut input,
        "holding_rate",
        text,
        &["持有成本率", "库存持有成本率", "holding_rate"],
    );
    insert_metric_value(
        &mut input,
        "lead_time_days",
        text,
        &["交货周期", "供货周期", "补货周期", "lead_time_days"],
    );
    if let Some(service_level) = infer_service_level(text) {
        input.insert(
            "service_level".to_string(),
            serde_json::json!(service_level),
        );
    }
    input
}

fn infer_inventory_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_inventory_sku_segment)
        .collect()
}

fn infer_ad_fatigue_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    let channels = infer_ad_fatigue_channels(text);
    if !channels.is_empty() {
        input.insert("channels".to_string(), serde_json::json!(channels));
    }
    insert_metric_value(
        &mut input,
        "total_budget",
        text,
        &["总预算", "广告总预算", "total_budget"],
    );
    input
}

fn infer_ad_fatigue_channels(text: &str) -> Vec<serde_json::Value> {
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
        if let Some(value) = ctr_baseline {
            row.insert("ctr_baseline".to_string(), serde_json::json!(value));
        }
        if let Some(value) = ctr_recent {
            row.insert("ctr_recent".to_string(), serde_json::json!(value));
        }
        if let Some(value) = impressions_per_user {
            row.insert("impressions_per_user".to_string(), serde_json::json!(value));
        }
        if let Some(value) = budget {
            row.insert("budget".to_string(), serde_json::json!(value));
        }
        if let Some(value) = roas {
            row.insert("roas".to_string(), serde_json::json!(value));
        }
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
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

fn infer_abc_xyz_classifier_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    let skus = infer_abc_xyz_skus(text);
    if !skus.is_empty() {
        input.insert("skus".to_string(), serde_json::json!(skus));
    }
    if let Some(top_n) = extract_metric_value(text, &["top_n", "前N", "前n", "展示前"]) {
        input.insert(
            "top_n".to_string(),
            serde_json::json!(top_n.round().clamp(1.0, 100.0) as i64),
        );
    }
    input
}

fn infer_abc_xyz_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_abc_xyz_sku_segment)
        .collect()
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

fn infer_abc_xyz_sku_id(segment: &str) -> String {
    let trimmed = segment
        .trim()
        .trim_start_matches(['-', '•', '*', ' '])
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    let trimmed = lower
        .rfind("sku")
        .map(|index| trimmed[index ..].trim())
        .unwrap_or(trimmed);
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
    let lower = trimmed.to_ascii_lowercase();
    let end = metric_markers
        .iter()
        .filter_map(|marker| lower.find(&marker.to_ascii_lowercase()))
        .min()
        .unwrap_or(trimmed.len());
    let raw = trimmed[.. end]
        .trim()
        .trim_matches(['：', ':', ',', '，', ';', '；', ' '])
        .trim();
    if raw.is_empty() {
        "SKU".to_string()
    } else {
        raw.to_string()
    }
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

fn infer_inventory_sku_name(segment: &str) -> String {
    let trimmed = segment
        .trim()
        .trim_start_matches(['-', '•', '*', ' '])
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    let trimmed = lower
        .rfind("sku")
        .map(|index| trimmed[index ..].trim())
        .unwrap_or(trimmed);
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
    let lower = trimmed.to_ascii_lowercase();
    let end = metric_markers
        .iter()
        .filter_map(|marker| {
            let marker_lower = marker.to_ascii_lowercase();
            lower.find(&marker_lower)
        })
        .min()
        .unwrap_or(trimmed.len());
    let raw = trimmed[.. end]
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
        "SKU".to_string()
    } else {
        raw.to_string()
    }
}

fn infer_service_level(text: &str) -> Option<String> {
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

