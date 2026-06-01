use super::{
    context, domain,
    metrics::{extract_metric_value, infer_metric},
    numbers::{infer_period_days, numbers_in_text},
};
use crate::agent::worker::{Task, runner::policy::worker_policies};

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
    extract_metric_value(text, &aliases("cost"))
        .unwrap_or(fallback)
        .max(number_default("rules/cost/result_min", 0.01))
}

pub(in crate::agent::worker::runner) fn infer_current_price(text: &str, fallback: f64) -> f64 {
    extract_metric_value(text, &aliases("current_price"))
        .unwrap_or(fallback)
        .max(number_default("rules/current_price/result_min", 0.0))
}

pub(in crate::agent::worker::runner) fn infer_stock_days(text: &str, fallback: f64) -> f64 {
    extract_metric_value(text, &aliases("stock_days"))
        .unwrap_or(fallback)
        .max(number_default("rules/stock_days/result_min", 0.0))
}

pub(in crate::agent::worker::runner) fn infer_elasticity(text: &str, fallback: f64) -> f64 {
    let value = extract_metric_value(text, &aliases("elasticity")).unwrap_or(fallback);
    if bool_default("rules/elasticity/positive_values_are_negated", true) && value > 0.0 {
        -value
    } else {
        value
    }
}

pub(in crate::agent::worker::runner) fn infer_pricing_strategy_competitor_prices(
    text: &str,
    fallback_cost: f64,
) -> Vec<f64> {
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

pub(in crate::agent::worker::runner) fn infer_budget(
    text: &str,
    aliases: &[impl AsRef<str>],
    default: f64,
) -> f64 {
    let numbers = numbers_in_text(text);
    let excluded = integer_values("rules/budget/excluded_integer_values", &[618, 11]);
    extract_metric_value(text, aliases)
        .or_else(|| {
            numbers
                .into_iter()
                .filter(|value| !excluded.contains(&(*value as i64)))
                .find(|value| *value >= number_default("rules/budget/number_min", 100.0))
        })
        .unwrap_or(default)
        .max(number_default("rules/budget/result_min", 0.0))
}

pub(in crate::agent::worker::runner) fn infer_default_budget(text: &str) -> f64 {
    infer_budget(
        text,
        &aliases("budget"),
        number_default("defaults/default_budget", 10_000.0),
    )
}

pub(in crate::agent::worker::runner) fn infer_demand_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
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
            &aliases("channel_strategy_budget"),
            number_default("defaults/channel_strategy_budget", 10_000.0),
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
            &aliases("assortment_planning_budget"),
            number_default("defaults/assortment_planning_budget", 50_000.0),
        )),
    );
    input
}

fn parse_price_quantity_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return None;
    }
    let price = extract_last_metric_value(trimmed, &aliases("price_quantity_point_price"))?;
    let quantity = extract_metric_value(trimmed, &aliases("price_quantity_point_quantity"))?;
    Some(serde_json::json!({
        "price": price,
        "quantity": quantity,
    }))
}

fn extract_last_metric_value<T: AsRef<str>>(text: &str, aliases: &[T]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias = alias.as_ref();
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

fn aliases(key: &str) -> Vec<String> {
    worker_policies()
        .inference
        .string_list("aliases", &format!("/{}", key))
}

fn number_default(pointer: &str, fallback: f64) -> f64 {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback;
    };
    worker_policies()
        .inference
        .f64_or(section, &format!("/{}", nested), fallback)
}

fn bool_default(pointer: &str, fallback: bool) -> bool {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback;
    };
    worker_policies()
        .inference
        .section(section)
        .and_then(|value| value.pointer(&format!("/{}", nested)))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(fallback)
}

fn integer_values(pointer: &str, fallback: &[i64]) -> Vec<i64> {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback.to_vec();
    };
    worker_policies()
        .inference
        .section(section)
        .and_then(|value| value.pointer(&format!("/{}", nested)))
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(serde_json::Value::as_i64).collect())
        .unwrap_or_else(|| fallback.to_vec())
}
