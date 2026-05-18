use serde_json::{Map, Value};

pub(crate) fn compact_text_for_task_title(value: &str) -> String {
    compact_task_text_value(value, 255)
}

pub(crate) fn compact_text_for_task_body(value: &str) -> String {
    compact_task_text_value(value, 2_000)
}

pub(crate) fn compact_task_text_value(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect()
}

pub(crate) fn metadata_string_value(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn metadata_u64_value(metadata: &Map<String, Value>, key: &str) -> Option<u64> {
    metadata.get(key).and_then(Value::as_u64)
}

pub(crate) fn metadata_non_empty(metadata: &Map<String, Value>, key: &str) -> bool {
    metadata_string_value(metadata, key).is_some()
}

pub(crate) fn metadata_csv_list(metadata: &Map<String, Value>, key: &str) -> Vec<String> {
    metadata_string_value(metadata, key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn infer_primary_product_id(metadata: &Map<String, Value>) -> Option<String> {
    metadata_csv_list(metadata, "context.product_ids")
        .into_iter()
        .next()
        .or_else(|| {
            metadata_csv_list(metadata, "context.refs")
                .into_iter()
                .find_map(|item| product_id_from_ref(&item).map(ToOwned::to_owned))
        })
}

pub(crate) fn infer_primary_product_ref(metadata: &Map<String, Value>) -> Option<String> {
    metadata_csv_list(metadata, "context.refs")
        .into_iter()
        .find(|item| product_id_from_ref(item).is_some())
        .or_else(|| {
            metadata_csv_list(metadata, "context.product_ids")
                .into_iter()
                .next()
                .map(|product_id| format!("product:{product_id}"))
        })
}

pub(crate) fn product_id_from_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("product:")
        .or_else(|| value.strip_prefix("product/"))
        .filter(|value| !value.trim().is_empty())
}
