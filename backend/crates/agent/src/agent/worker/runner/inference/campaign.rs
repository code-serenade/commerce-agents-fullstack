fn infer_douyin_ads_plan_structure_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if first_selected_product(task).is_some()
        || contains_any(text, &["商品", "产品", "咖啡", "SKU", "sku"])
    {
        input.insert(
            "product_name".to_string(),
            serde_json::json!(selected_product_name_or_task(task)),
        );
    }
    input.insert(
        "goal".to_string(),
        serde_json::json!(infer_douyin_ads_goal(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_explicit_period_days(text).unwrap_or(7)),
    );
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "投放预算",
            "广告预算",
            "千川预算",
            "总预算",
            "预算",
            "budget",
        ],
    );
    insert_metric_value(
        &mut input,
        "target_roas",
        text,
        &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
    );
    input
}

fn infer_douyin_ads_audience_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if first_selected_product(task).is_some()
        || contains_any(text, &["商品", "产品", "咖啡", "SKU", "sku"])
    {
        input.insert(
            "product_name".to_string(),
            serde_json::json!(selected_product_name_or_task(task)),
        );
    }
    input.insert(
        "target_audience".to_string(),
        serde_json::json!(infer_audience(text)),
    );
    input.insert(
        "category".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "人群预算",
            "投放预算",
            "广告预算",
            "千川预算",
            "预算",
            "budget",
        ],
    );
    input
}

fn infer_douyin_ads_budget_pacing_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_explicit_period_days(text).unwrap_or(7)),
    );
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "投放预算",
            "广告预算",
            "千川预算",
            "总预算",
            "预算",
            "budget",
        ],
    );
    insert_metric_value(
        &mut input,
        "spend",
        text,
        &[
            "已消耗",
            "消耗",
            "花费",
            "广告费",
            "广告花费",
            "spend",
            "ad_spend",
        ],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "成交额", "销售额", "营收"],
    );
    if let Some(current_roas) = infer_douyin_ads_current_roas(text) {
        input.insert("current_roas".to_string(), serde_json::json!(current_roas));
    }
    insert_metric_value(
        &mut input,
        "target_roas",
        text,
        &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
    );
    input
}

fn infer_douyin_ads_live_funnel_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "impressions",
        text,
        &["曝光量", "曝光", "展现", "展示", "impressions", "exposure"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "广告点击", "clicks", "click"],
    );
    insert_metric_value(
        &mut input,
        "live_room_entries",
        text,
        &["进房数", "进房", "进入直播间", "场观", "live_room_entries"],
    );
    insert_metric_value(
        &mut input,
        "product_clicks",
        text,
        &["商品点击数", "商品点击", "点击商品", "product_clicks"],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &["下单数", "下单", "订单数", "订单", "orders"],
    );
    insert_metric_value(
        &mut input,
        "payments",
        text,
        &["付款数", "付款", "支付数", "支付", "payments"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告消耗",
            "广告花费",
            "广告费",
            "消耗",
            "spend",
            "ad_spend",
        ],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "成交额", "销售额", "营收"],
    );
    input
}

fn infer_douyin_ads_goal(text: &str, task: &Task) -> String {
    if contains_any(text, &["直播", "直播间", "进房", "场观"]) {
        "直播间成交与进房转化".to_string()
    } else if contains_any(text, &["短视频", "视频", "素材"]) {
        "短视频引流与商品转化".to_string()
    } else if contains_any(text, &["新品", "冷启动", "起量"]) {
        "新品冷启动起量".to_string()
    } else {
        compact_task_text(&task.description)
    }
}

fn infer_douyin_ads_current_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &[
            "当前ROAS",
            "当前roas",
            "实际ROAS",
            "实际roas",
            "当前投产",
            "实际投产",
            "当前ROI",
            "当前roi",
            "实际ROI",
            "实际roi",
        ],
    )
    .or_else(|| {
        if contains_any(
            text,
            &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
        ) {
            None
        } else {
            extract_metric_value(text, &["ROAS", "roas", "投产", "ROI", "roi"])
        }
    })
}

fn infer_promo_budget(text: &str) -> Option<f64> {
    extract_metric_value(text, &["活动预算", "总预算", "预算", "budget"])
        .filter(|value| *value >= 100.0)
}

fn infer_competitor_prices(text: &str) -> Vec<f64> {
    let mut prices = text
        .split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(|segment| {
            let trimmed = segment.trim();
            if !contains_any(
                trimmed,
                &["竞品", "对手", "竞对", "同行", "competitor", "competing"],
            ) {
                return None;
            }
            extract_last_metric_value(
                trimmed,
                &[
                    "竞品价格",
                    "竞品价",
                    "对手价格",
                    "竞对价格",
                    "同行价格",
                    "价格",
                    "售价",
                    "price",
                ],
            )
        })
        .collect::<Vec<_>>();
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    prices.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    prices
}

