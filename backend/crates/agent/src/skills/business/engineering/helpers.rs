use serde_json::{Map, Value};

pub(super) fn str_field(input: &Map<String, Value>, key: &str, default: &str) -> String {
    opt_str_field(input, key).unwrap_or_else(|| default.to_owned())
}

pub(super) fn opt_str_field(input: &Map<String, Value>, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn string_vec_field(input: &Map<String, Value>, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn i64_field(input: &Map<String, Value>, key: &str) -> Option<i64> {
    input.get(key).and_then(Value::as_i64)
}

pub(super) fn f64_field(input: &Map<String, Value>, key: &str) -> Option<f64> {
    input.get(key).and_then(Value::as_f64)
}

pub(super) fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
