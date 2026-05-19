use super::numbers::numbers_in_text;
use crate::agent::worker::runner::policy::worker_policies;

pub(in crate::agent::worker::runner) fn infer_store_metrics_platform(text: &str) -> String {
    worker_policies()
        .inference
        .match_label(text, "platform.store_metrics")
        .unwrap_or("all")
        .to_string()
}

pub(in crate::agent::worker::runner) fn infer_platform(text: &str) -> String {
    worker_policies()
        .inference
        .match_label(text, "platform.default")
        .unwrap_or_else(|| default_str("platform_default", ""))
        .to_string()
}

pub(in crate::agent::worker::runner) fn infer_stage(text: &str) -> String {
    worker_policies()
        .inference
        .match_label(text, "stage")
        .unwrap_or_else(|| default_str("stage", ""))
        .to_string()
}

pub(in crate::agent::worker::runner) fn infer_metric(text: &str) -> &'static str {
    worker_policies()
        .inference
        .match_label(text, "metric.default")
        .unwrap_or_else(|| default_str("metric_default", "gmv"))
}

pub(super) fn extract_metric_value<T: AsRef<str>>(text: &str, aliases: &[T]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias = alias.as_ref();
        let alias_lower = alias.to_ascii_lowercase();
        let Some(index) = lower.find(&alias_lower) else {
            continue;
        };
        let start = index + alias_lower.len();
        if let Some(value) = parse_metric_number_after_alias(&text[start ..]) {
            return Some(value);
        }
    }
    None
}

pub(super) fn first_number(text: &str) -> Option<f64> {
    numbers_in_text(text).first().copied()
}

pub(in crate::agent::worker::runner) fn infer_cohort_unit(text: &str) -> &'static str {
    worker_policies()
        .inference
        .match_label(text, "cohort.unit")
        .unwrap_or_else(|| default_str("cohort_unit", "week"))
}

pub(in crate::agent::worker::runner) fn infer_seasonal_period(text: &str) -> i64 {
    worker_policies()
        .inference
        .match_label(text, "seasonal.period")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_else(|| default_i64("seasonal_period", 7))
}

pub(super) fn parse_metric_number_after_alias(segment: &str) -> Option<f64> {
    let mut chars = segment.char_indices().peekable();
    let mut number_start = None;
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_ascii_digit() || ch == '.' {
            number_start = Some(index);
            break;
        }
        if matches!(
            ch,
            ':' | '：' | '=' | '＝' | ' ' | '\t' | '约' | '为' | '是' | '￥' | '¥'
        ) {
            chars.next();
            continue;
        }
        if index > 12 {
            return None;
        }
        chars.next();
    }

    let start = number_start?;
    let mut end = start;
    let mut dot_seen = false;
    for (index, ch) in segment[start ..].char_indices() {
        if ch.is_ascii_digit() || ch == ',' {
            end = start + index + ch.len_utf8();
            continue;
        }
        if ch == '.' && !dot_seen {
            dot_seen = true;
            end = start + index + ch.len_utf8();
            continue;
        }
        break;
    }

    let raw_number = segment[start .. end].replace(',', "");
    let mut value = raw_number.parse::<f64>().ok()?;
    if !value.is_finite() {
        return None;
    }

    let unit_tail = segment[end ..].trim_start();
    if unit_tail.starts_with('亿') {
        value *= 100_000_000.0;
    } else if unit_tail.starts_with('万')
        || unit_tail.starts_with('w')
        || unit_tail.starts_with('W')
    {
        value *= 10_000.0;
    } else if unit_tail.starts_with('千')
        || unit_tail.starts_with('k')
        || unit_tail.starts_with('K')
    {
        value *= 1_000.0;
    }
    Some(value.max(0.0))
}

fn default_str(key: &str, fallback: &'static str) -> &'static str {
    worker_policies()
        .inference
        .section("defaults")
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
}

fn default_i64(key: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .section("defaults")
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(fallback)
}
