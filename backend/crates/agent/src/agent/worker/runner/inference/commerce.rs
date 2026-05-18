use super::{
    context, domain,
    metrics::{extract_metric_value, infer_metric},
    numbers::{infer_period_days, numbers_in_text},
};
use crate::agent::worker::Task;

pub(in crate::agent::worker::runner) fn infer_price_quantity_points(
    text: &str,
) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_price_quantity_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_competitor_prices(text: &str) -> Vec<f64> {
    let values = numbers_in_text(text);
    values
        .into_iter()
        .filter(|value| *value >= 1.0 && *value <= 100_000.0)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_cost(text: &str, fallback: f64) -> f64 {
    extract_metric_value(text, &["单位成本", "单品成本", "成本", "cost"])
        .unwrap_or(fallback)
        .max(0.01)
}

pub(in crate::agent::worker::runner) fn infer_current_price(text: &str, fallback: f64) -> f64 {
    extract_metric_value(
        text,
        &[
            "当前售价",
            "当前价格",
            "现价",
            "售价",
            "current_price",
            "price",
        ],
    )
    .unwrap_or(fallback)
    .max(0.0)
}

pub(in crate::agent::worker::runner) fn infer_budget(text: &str, aliases: &[&str], default: f64) -> f64 {
    let numbers = numbers_in_text(text);
    extract_metric_value(text, aliases)
        .or_else(|| {
            numbers
                .into_iter()
                .filter(|value| !matches!(*value as i64, 618 | 11))
                .find(|value| *value >= 100.0)
        })
        .unwrap_or(default)
        .max(0.0)
}

pub(in crate::agent::worker::runner) fn infer_demand_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert("days".to_string(), serde_json::json!(infer_period_days(text)));
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    if let Some(product_ref) = context::selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

pub(in crate::agent::worker::runner) fn infer_promo_planning_input(
    text: &str,
    task: &Task,
    budget: f64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "product".to_string(),
        serde_json::json!(context::selected_product_name_or_task(task)),
    );
    if let Some(product_ref) = context::selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert("budget".to_string(), serde_json::json!(budget));
    input.insert(
        "platform".to_string(),
        serde_json::json!(context::selected_product_platform_or_infer(task, text)),
    );
    input
}

pub(in crate::agent::worker::runner) fn infer_channel_strategy_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "product_type".to_string(),
        serde_json::json!(context::selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = context::selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(infer_budget(
            text,
            &["渠道预算", "月预算", "预算", "budget"],
            10_000.0,
        )),
    );
    input.insert(
        "target_audience".to_string(),
        serde_json::json!(domain::infer_audience(text)),
    );
    input
}

pub(in crate::agent::worker::runner) fn infer_assortment_planning_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "category".to_string(),
        serde_json::json!(context::selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = context::selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(infer_budget(
            text,
            &["选品预算", "货盘预算", "采购预算", "预算", "budget"],
            50_000.0,
        )),
    );
    input
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
        if let Some(value) = super::metrics::parse_metric_number_after_alias(&text[start..]) {
            return Some(value);
        }
    }
    None
}
