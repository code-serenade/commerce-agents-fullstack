use super::text::compact_task_text;
use crate::agent::worker::{Task, runner::policy::worker_policies};

pub(in crate::agent::worker::runner) fn infer_memory_key(text: &str) -> String {
    let compact = compact_task_text(text);
    let mut key = compact
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    while key.contains("__") {
        key = key.replace("__", "_");
    }
    key = key.trim_matches('_').to_string();
    if key.is_empty() {
        config_string("/memory_key/empty_fallback", "workspace_memory")
    } else {
        key.chars()
            .take(config_i64("/memory_key/max_chars", 80).max(0) as usize)
            .collect()
    }
}

pub(in crate::agent::worker::runner) fn infer_memory_type(text: &str) -> &'static str {
    classify_string(
        text,
        "memory_type",
        config_string("/defaults/memory_type", "fact"),
    )
    .leak()
}

pub(in crate::agent::worker::runner) fn infer_memory_content(text: &str, task: &Task) -> String {
    let compact = compact_task_text(text);
    let max_chars = config_i64("/content/max_chars", 500).max(0) as usize;
    if compact.chars().count() >= config_i64("/content/compact_min_chars", 24).max(0) as usize {
        return compact.chars().take(max_chars).collect();
    }
    compact_task_text(&task.description)
        .chars()
        .take(max_chars)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_task_title(text: &str, task: &Task) -> String {
    extract_title_candidate(text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| compact_task_text(&task.description))
}

pub(in crate::agent::worker::runner) fn infer_task_owner_role(text: &str) -> &'static str {
    classify_string(
        text,
        "task_owner_role",
        config_string("/defaults/task_owner_role", "ops"),
    )
    .leak()
}

pub(in crate::agent::worker::runner) fn infer_task_priority(text: &str) -> i64 {
    classify_i64(
        text,
        "task_priority",
        config_i64("/defaults/task_priority", 3),
    )
}

pub(in crate::agent::worker::runner) fn infer_task_acceptance(text: &str) -> String {
    for marker in config_string_list("/task_acceptance/section_markers") {
        if let Some(index) = text.find(&marker) {
            return compact_task_text(&text[index..]);
        }
    }
    config_string("/defaults/task_acceptance", "产出明确结果并可在工作区复用")
}

fn extract_title_candidate(text: &str) -> Option<String> {
    for quote in config_char_list("/title_candidate/quote_delimiters") {
        let mut parts = text.split(quote);
        let _ = parts.next();
        if let Some(candidate) = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(compact_task_text(candidate));
        }
    }
    None
}

fn classify_string(text: &str, classifier: &str, fallback: String) -> String {
    classify_value(text, classifier)
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or(fallback)
}

fn classify_i64(text: &str, classifier: &str, fallback: i64) -> i64 {
    classify_value(text, classifier)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(fallback)
}

fn classify_value<'a>(text: &str, classifier: &str) -> Option<&'a serde_json::Value> {
    section()
        .pointer(&format!("/classifiers/{classifier}"))?
        .as_array()?
        .iter()
        .find_map(|rule| {
            let keywords = rule.get("keywords")?.as_array()?;
            let matched = keywords
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|keyword| contains_keyword(text, keyword));
            matched.then(|| rule.get("value")).flatten()
        })
}

fn contains_keyword(text: &str, needle: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("memory_task", pointer, fallback)
}

fn config_string(pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or("memory_task", pointer, fallback)
}

fn config_string_list(pointer: &str) -> Vec<String> {
    worker_policies()
        .inference
        .string_list("memory_task", pointer)
}

fn config_char_list(pointer: &str) -> Vec<char> {
    config_string_list(pointer)
        .into_iter()
        .filter_map(|value| {
            let mut chars = value.chars();
            let ch = chars.next()?;
            chars.next().is_none().then_some(ch)
        })
        .collect()
}

fn section() -> &'static serde_json::Value {
    worker_policies()
        .inference
        .section("memory_task")
        .unwrap_or(&serde_json::Value::Null)
}
