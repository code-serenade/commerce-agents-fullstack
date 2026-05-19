use serde_json::{Map, Value};

use super::output::{metadata_string_value, push_unique_non_empty};

pub(super) fn collect_metadata_texts(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        if let Some(value) = metadata_string_value(metadata, key) {
            push_unique_non_empty(&mut out, value);
        }
    }
    out
}

pub(super) fn collect_metadata_text_parts(
    metadata: &Map<String, Value>,
    keys: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    for value in collect_metadata_texts(metadata, keys) {
        for part in value.split('|') {
            push_unique_non_empty(&mut out, part.trim().to_owned());
        }
    }
    out
}

pub(super) fn collect_metadata_list(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_list_value(&mut out, &parsed);
                } else {
                    for item in text.split(',') {
                        push_unique_non_empty(&mut out, item.trim().to_owned());
                    }
                }
            }
            other => collect_list_value(&mut out, other),
        }
    }
    out
}

pub(super) fn collect_metadata_sources(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<Value> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_source_value(&mut out, parsed);
                }
            }
            other => collect_source_value(&mut out, other.clone()),
        }
    }
    out
}

fn collect_list_value(out: &mut Vec<String>, value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_list_value(out, item);
            }
        }
        Value::String(text) => {
            for item in text.split(',') {
                push_unique_non_empty(out, item.trim().to_owned());
            }
        }
        Value::Number(number) => push_unique_non_empty(out, number.to_string()),
        Value::Bool(value) => push_unique_non_empty(out, value.to_string()),
        Value::Object(_) | Value::Null => {}
    }
}

fn collect_source_value(out: &mut Vec<Value>, value: Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_source_value(out, item);
            }
        }
        Value::Object(_) | Value::String(_) => {
            if !out.iter().any(|item| item == &value) {
                out.push(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}
