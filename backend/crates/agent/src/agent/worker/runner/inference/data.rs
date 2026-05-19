use super::{
    metrics::extract_metric_value,
    numbers::{infer_period_days, numbers_in_text},
};
use crate::agent::worker::runner::policy::worker_policies;

pub(in crate::agent::worker::runner) fn infer_values(text: &str) -> Vec<f64> {
    numbers_in_text(text)
}

pub(in crate::agent::worker::runner) fn infer_channel_roi_rows(
    text: &str,
) -> Vec<serde_json::Value> {
    let platform_labels = string_list("platforms", "/channel_roi_labels");
    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in &platform_labels {
        let Some(segment) = segment_after_label(text, label, &platform_labels) else {
            continue;
        };
        let name = canonical_channel_name(label);
        if seen.iter().any(|value| value == &name) {
            continue;
        }

        let gmv = extract_metric_value(&segment, &aliases("channel_roi", "gmv"));
        let ad_spend = extract_metric_value(&segment, &aliases("channel_roi", "ad_spend"));
        let orders = extract_metric_value(&segment, &aliases("channel_roi", "orders"));
        let uv = extract_metric_value(&segment, &aliases("channel_roi", "uv"));
        let conversion_rate =
            extract_metric_value(&segment, &aliases("channel_roi", "conversion_rate"));

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
        seen.push(name);
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

pub(in crate::agent::worker::runner) fn infer_attribution_model(text: &str) -> &'static str {
    classify_or(
        text,
        "attribution_model",
        default_str("attribution_model", "all"),
    )
}

pub(in crate::agent::worker::runner) fn infer_refund_items(text: &str) -> Vec<serde_json::Value> {
    let separators = string_list("parsers", "/refund_items/split_on");
    text.split(|ch: char| {
        separators
            .iter()
            .any(|separator| separator == &ch.to_string())
    })
    .filter_map(parse_refund_item_segment)
    .collect()
}

pub(in crate::agent::worker::runner) fn infer_metric_number(
    text: &str,
    metric: &str,
) -> Option<f64> {
    extract_metric_value(text, &aliases("metric_numbers", metric)).map(|value| {
        if metric == "forecast_periods" {
            clamp_rule(value.round(), "forecast_periods", 1.0, 90.0)
        } else {
            value
        }
    })
}

pub(in crate::agent::worker::runner) fn infer_metric_integer(
    text: &str,
    metric: &str,
) -> Option<i64> {
    infer_metric_number(text, metric).map(|value| {
        let min = rules_value()
            .pointer("/metric_integer/min")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        value.round().max(min) as i64
    })
}

pub(in crate::agent::worker::runner) fn infer_ab_integer(text: &str, key: &str) -> Option<i64> {
    extract_metric_value(text, &aliases("ab_integers", key))
        .map(|value| value.round().max(0.0) as i64)
        .or_else(|| infer_ab_number_fallback(text, key))
}

fn infer_ab_number_fallback(text: &str, key: &str) -> Option<i64> {
    let numbers = numbers_in_text(text);
    let minimum_number_count = rules_value()
        .pointer("/ab_number_fallback/minimum_number_count")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(4);
    if numbers.len() < minimum_number_count {
        return None;
    }
    let index = rules_value()
        .pointer(&format!("/ab_number_fallback/index_by_key/{key}"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())?;
    numbers
        .get(index)
        .map(|value| value.round().max(0.0) as i64)
}

pub(in crate::agent::worker::runner) fn infer_ab_test_name(text: &str) -> &'static str {
    classify_or(text, "ab_test_name", default_str("ab_test_name", "A/B测试"))
}

pub(in crate::agent::worker::runner) fn infer_ab_metric(text: &str) -> &'static str {
    classify_or(text, "ab_metric", default_str("ab_metric", "转化率"))
}

pub(in crate::agent::worker::runner) fn infer_multi_periods(text: &str) -> Vec<i64> {
    classify_i64_list(text, "multi_periods")
        .unwrap_or_else(|| i64_list_or("defaults", "/multi_periods", &[7, 14, 30]))
}

pub(in crate::agent::worker::runner) fn infer_forecast_periods(text: &str) -> i64 {
    infer_metric_number(text, "forecast_periods")
        .map(|value| clamp_rule(value.round(), "forecast_periods", 1.0, 90.0) as i64)
        .unwrap_or_else(|| default_i64("forecast_periods", 7))
}

pub(in crate::agent::worker::runner) fn infer_top_n(text: &str, default: i64) -> i64 {
    infer_metric_number(text, "top_n")
        .map(|value| clamp_rule(value.round(), "top_n", 1.0, 100.0) as i64)
        .unwrap_or(default)
}

pub(in crate::agent::worker::runner) fn infer_diagnosis_focus(text: &str) -> &'static str {
    classify_or(
        text,
        "diagnosis_focus",
        default_str("diagnosis_focus", "all"),
    )
}

pub(in crate::agent::worker::runner) fn infer_labeled_token(
    text: &str,
    token: &str,
) -> Option<String> {
    let aliases = aliases("labeled_tokens", token);
    let alias_refs = aliases.iter().map(String::as_str).collect::<Vec<_>>();
    super::domain::extract_labeled_token(text, &alias_refs)
}

pub(in crate::agent::worker::runner) fn default_days(text: &str) -> i64 {
    infer_period_days(text)
}

fn parse_refund_item_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.rsplit(['：', ':']).next().unwrap_or(segment).trim();
    if trimmed.is_empty() {
        return None;
    }
    let refund_aliases = refund_amount_aliases();
    let amount = extract_metric_value(trimmed, &refund_aliases)?;
    let name = refund_item_name(trimmed, &refund_aliases)
        .map(str::to_string)
        .unwrap_or_else(|| default_string("refund_item_name", "综合退款"));
    Some(serde_json::json!({
        "name": name,
        "refund_amount": amount,
    }))
}

fn refund_item_name<'a>(segment: &'a str, aliases: &[String]) -> Option<&'a str> {
    for alias in aliases {
        if let Some(index) = segment.find(alias) {
            let name = segment[.. index]
                .trim()
                .trim_matches(['：', ':', '-', '，', ',', '；', ';', ' ']);
            return if name.is_empty() { None } else { Some(name) };
        }
    }
    None
}

fn segment_after_label(text: &str, label: &str, labels: &[String]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();
    let index = lower.find(&label_lower)?;
    let start = index + label.len();
    let mut end = text.len();

    for other in labels {
        if other == label {
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

fn canonical_channel_name(label: &str) -> String {
    if let Some(channel) = policy_section("platforms")
        .pointer("/canonical_channels")
        .and_then(serde_json::Value::as_array)
        .and_then(|channels| {
            channels.iter().find_map(|channel| {
                let aliases = channel.get("aliases")?.as_array()?;
                let matched = aliases
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|alias| alias == label || alias.eq_ignore_ascii_case(label));
                matched.then(|| channel.get("label")?.as_str()).flatten()
            })
        })
    {
        return channel.to_string();
    }

    string_or("platforms", "/unknown_channel", "未知渠道")
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

fn aliases(group: &str, key: &str) -> Vec<String> {
    string_list("aliases", &format!("/{group}/{key}"))
}

fn refund_amount_aliases() -> Vec<String> {
    string_list("aliases", "/refund_amount")
}

fn classify_or(text: &str, classifier: &str, fallback: &'static str) -> &'static str {
    classify(text, classifier).unwrap_or(fallback)
}

fn classify(text: &str, classifier: &str) -> Option<&'static str> {
    policy_section("classifiers")
        .get(classifier)?
        .as_array()?
        .iter()
        .find_map(|rule| {
            let value = rule.get("value")?.as_str()?;
            let keywords = rule.get("keywords")?.as_array()?;
            let matched = keywords
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|keyword| contains_keyword(text, keyword));
            matched.then_some(value)
        })
}

fn classify_i64_list(text: &str, classifier: &str) -> Option<Vec<i64>> {
    policy_section("classifiers")
        .get(classifier)?
        .as_array()?
        .iter()
        .find_map(|rule| {
            let keywords = rule.get("keywords")?.as_array()?;
            let matched = keywords
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|keyword| contains_keyword(text, keyword));
            matched.then(|| {
                rule.get("value")?
                    .as_array()?
                    .iter()
                    .map(serde_json::Value::as_i64)
                    .collect::<Option<Vec<_>>>()
            })?
        })
}

fn clamp_rule(value: f64, rule: &str, fallback_min: f64, fallback_max: f64) -> f64 {
    let rules = rules_value();
    let min = rules
        .pointer(&format!("/{rule}/min"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(fallback_min);
    let max = rules
        .pointer(&format!("/{rule}/max"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(fallback_max);
    value.clamp(min, max)
}

fn default_str(key: &str, fallback: &'static str) -> &'static str {
    policy_section("defaults")
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
}

fn default_string(key: &str, fallback: &str) -> String {
    policy_section("defaults")
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn default_i64(key: &str, fallback: i64) -> i64 {
    policy_section("defaults")
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(fallback)
}

fn string_or(section: &str, pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or(section, pointer, fallback)
}

fn string_list(section: &str, pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list(section, pointer)
}

fn i64_list_or(section: &str, pointer: &str, fallback: &[i64]) -> Vec<i64> {
    policy_section(section)
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_i64)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| fallback.to_vec())
}

fn policy_section(section: &str) -> &'static serde_json::Value {
    worker_policies()
        .inference
        .section(section)
        .unwrap_or(&serde_json::Value::Null)
}

fn rules_value() -> &'static serde_json::Value {
    policy_section("rules")
}

fn contains_keyword(text: &str, needle: &str) -> bool {
    text.contains(needle)
        || text
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
}
