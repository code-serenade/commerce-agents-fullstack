fn infer_price_elasticity_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let price_points = infer_price_quantity_points(text);
    if !price_points.is_empty() {
        input.insert("price_points".to_string(), serde_json::json!(price_points));
    }
    insert_metric_value(&mut input, "cost", text, &["成本", "单品成本", "cost"]);
    insert_metric_value(
        &mut input,
        "current_price",
        text,
        &["当前售价", "当前价格", "现价", "current_price"],
    );
    input
}

fn infer_smart_pricing_input(
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
    insert_metric_value(
        &mut input,
        "cost",
        text,
        &["单位成本", "单品成本", "成本", "cost"],
    );
    insert_metric_value(
        &mut input,
        "current_price",
        text,
        &[
            "当前售价",
            "当前价格",
            "现价",
            "售价",
            "current_price",
            "price",
        ],
    );
    insert_metric_value(
        &mut input,
        "current_monthly_sales",
        text,
        &[
            "当前月销量",
            "月销量",
            "monthly_sales",
            "current_monthly_sales",
        ],
    );
    insert_metric_value(
        &mut input,
        "stock_days",
        text,
        &["库存天数", "可售天数", "stock_days"],
    );
    insert_metric_value(
        &mut input,
        "elasticity",
        text,
        &["弹性系数", "价格弹性", "elasticity"],
    );
    if let Some(value) = input.get("elasticity").and_then(serde_json::Value::as_f64)
        && value > 0.0
    {
        input.insert("elasticity".to_string(), serde_json::json!(-value));
    }
    let competitor_prices = infer_competitor_prices(text);
    if !competitor_prices.is_empty() {
        input.insert(
            "competitor_prices".to_string(),
            serde_json::json!(competitor_prices),
        );
    }
    input
}

fn infer_pricing_strategy_input(
    text: &str,
    task: &Task,
    fallback_cost: f64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "cost".to_string(),
        serde_json::json!(
            extract_metric_value(text, &["单位成本", "单品成本", "成本", "cost"])
                .unwrap_or(fallback_cost)
                .max(0.01)
        ),
    );
    input.insert(
        "competitor_prices".to_string(),
        serde_json::json!(infer_pricing_strategy_competitor_prices(
            text,
            fallback_cost
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}

fn infer_pricing_strategy_competitor_prices(text: &str, fallback_cost: f64) -> Vec<f64> {
    let prices = infer_competitor_prices(text);
    if prices.is_empty() {
        numbers_in_text(text)
            .into_iter()
            .filter(|value| (*value - fallback_cost).abs() > 0.01)
            .collect()
    } else {
        prices
    }
}

