fn infer_promo_planning_input(
    text: &str,
    task: &Task,
    budget: f64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "product".to_string(),
        serde_json::json!(selected_product_name_or_task(task)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert("budget".to_string(), serde_json::json!(budget));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}

fn infer_execution_plan_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "goal".to_string(),
        serde_json::json!(compact_task_text(&task.description)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input
}

fn infer_channel_strategy_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    input.insert(
        "product_type".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(
            extract_metric_value(text, &["渠道预算", "月预算", "预算", "budget"])
                .or_else(|| {
                    numbers
                        .into_iter()
                        .filter(|value| !matches!(*value as i64, 618 | 11))
                        .find(|value| *value >= 100.0)
                })
                .unwrap_or(10_000.0)
                .max(0.0)
        ),
    );
    input.insert(
        "target_audience".to_string(),
        serde_json::json!(infer_audience(text)),
    );
    input
}

fn infer_assortment_planning_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    input.insert(
        "category".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["选品预算", "货盘预算", "采购预算", "预算", "budget"]
            )
            .or_else(|| {
                numbers
                    .into_iter()
                    .filter(|value| !matches!(*value as i64, 618 | 11))
                    .find(|value| *value >= 100.0)
            })
            .unwrap_or(50_000.0)
            .max(0.0)
        ),
    );
    input
}

