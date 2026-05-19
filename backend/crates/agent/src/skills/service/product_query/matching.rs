use serde_json::Value;

use super::output::{eq_ignore_ascii_case_trimmed, strip_product_ref_prefix};

#[derive(Clone, Debug, Default)]
pub(super) struct ProductContextMatch {
    pub(super) matched: bool,
    pub(super) product_ids: Vec<String>,
    pub(super) refs: Vec<String>,
    pub(super) sources: Vec<Value>,
}

pub(super) fn matching_product_ids(product_ids: &[String], product_id: &str) -> Vec<String> {
    product_ids
        .iter()
        .filter(|item| eq_ignore_ascii_case_trimmed(item, product_id))
        .cloned()
        .collect()
}

pub(super) fn matching_product_ids_by_ref(
    product_ids: &[String],
    product_ref: &str,
) -> Vec<String> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    matching_product_ids(product_ids, product_id)
}

pub(super) fn matching_refs(refs: &[String], product_id: &str) -> Vec<String> {
    refs.iter()
        .filter(|item| text_contains_case_insensitive(item, product_id))
        .cloned()
        .collect()
}

pub(super) fn matching_sources_by_product_ref(sources: &[Value], product_ref: &str) -> Vec<Value> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_ref)
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

pub(super) fn matching_sources_by_product_id(sources: &[Value], product_id: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

pub(super) fn matching_sources_by_text(sources: &[Value], text: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| text_contains_case_insensitive(&source.to_string(), text))
        .cloned()
        .collect()
}

pub(super) fn source_product_id(source: &Value) -> Option<String> {
    source
        .get("product_id")
        .or_else(|| source.get("productId"))
        .and_then(|value| match value {
            Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
}

fn text_contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}
