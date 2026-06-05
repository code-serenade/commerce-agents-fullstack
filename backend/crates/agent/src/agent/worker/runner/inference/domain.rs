use super::{metrics::infer_platform, numbers::numbers_in_text, text::compact_task_text};
use crate::agent::worker::runner::policy::worker_policies;

pub(in crate::agent::worker::runner) fn infer_return_reason(text: &str) -> String {
    match_configured_label(text, "return_reason")
}

pub(in crate::agent::worker::runner) fn infer_service_issue_type(text: &str) -> String {
    match_configured_label(text, "service_issue_type")
}

pub(in crate::agent::worker::runner) fn infer_service_emotion(text: &str) -> String {
    match_configured_label(text, "service_emotion")
}

pub(in crate::agent::worker::runner) fn infer_design_usage(text: &str) -> String {
    match_configured_label(text, "design_usage")
}

pub(in crate::agent::worker::runner) fn infer_design_style(text: &str) -> String {
    match_configured_label(text, "design_style")
}

pub(in crate::agent::worker::runner) fn infer_page_type(text: &str) -> String {
    match_configured_label(text, "page_type")
}

pub(in crate::agent::worker::runner) fn infer_discount_hint(text: &str) -> String {
    match_configured_label(text, "discount_hint")
}

pub(in crate::agent::worker::runner) fn infer_video_type(text: &str) -> String {
    match_configured_label(text, "video_type")
}

pub(in crate::agent::worker::runner) fn infer_video_duration_seconds(text: &str) -> i64 {
    let numbers = numbers_in_text(text);
    if numbers.is_empty() {
        return config_i64("/video_duration_seconds/default", 30);
    }
    let min = config_i64("/video_duration_seconds/min", 10) as f64;
    let max = config_i64("/video_duration_seconds/max", 180) as f64;
    if contains_any(
        text,
        &config_string_list("/video_duration_seconds/second_keywords"),
    ) {
        return numbers[0].round().clamp(min, max) as i64;
    }
    if contains_any(
        text,
        &config_string_list("/video_duration_seconds/minute_keywords"),
    ) {
        return (numbers[0] * 60.0).round().clamp(min, max) as i64;
    }
    config_i64("/video_duration_seconds/default", 30)
}

pub(in crate::agent::worker::runner) fn infer_audience(text: &str) -> String {
    match_configured_label(text, "audience")
}

pub(in crate::agent::worker::runner) fn infer_category(text: &str) -> String {
    match_configured_label(text, "category")
}

pub(in crate::agent::worker::runner) fn infer_product_name(text: &str) -> String {
    compact_task_text(text)
}

pub(in crate::agent::worker::runner) fn infer_campaign_name(text: &str) -> String {
    match_configured_label_with_dynamic_default(text, "campaign_name", compact_task_text(text))
}

pub(in crate::agent::worker::runner) fn infer_campaign_type(text: &str) -> String {
    match_configured_label(text, "campaign_type")
}

pub(in crate::agent::worker::runner) fn extract_labeled_token(
    text: &str,
    labels: &[&str],
) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        let Some(index) = lower.find(&label_lower) else {
            continue;
        };
        let start = index + label_lower.len();
        if let Some(token) = parse_token_after_label(&text[start ..]) {
            return Some(token);
        }
    }
    None
}

pub(in crate::agent::worker::runner) fn infer_system_type(text: &str) -> String {
    match_configured_label(text, "system_type")
}

pub(in crate::agent::worker::runner) fn infer_bug_type(text: &str) -> String {
    match_configured_label(text, "bug_type")
}

pub(in crate::agent::worker::runner) fn infer_bug_frequency(text: &str) -> String {
    match_configured_label(text, "bug_frequency")
}

pub(in crate::agent::worker::runner) fn infer_perf_bottleneck(text: &str) -> String {
    match_configured_label(text, "perf_bottleneck")
}

pub(in crate::agent::worker::runner) fn infer_stack_hint(text: &str) -> String {
    let mut parts = Vec::new();
    for stack in config_string_list("/stack_hint/known_stacks") {
        if text.contains(&stack)
            || text
                .to_ascii_lowercase()
                .contains(&stack.to_ascii_lowercase())
        {
            parts.push(stack.to_string());
        }
    }
    if parts.is_empty() {
        config_string("/stack_hint/default", "未明确技术栈")
    } else {
        parts.join(&config_string("/stack_hint/separator", "/"))
    }
}

pub(in crate::agent::worker::runner) fn infer_tech_domain(text: &str) -> String {
    match_configured_label(text, "tech_domain")
}

pub(in crate::agent::worker::runner) fn infer_status_query_type(text: &str) -> String {
    match_configured_label(text, "status_query_type")
}

pub(in crate::agent::worker::runner) fn infer_platform_health_check_platform(text: &str) -> String {
    if !has_platform_health_check_intent(text) {
        return String::new();
    }
    let platform = infer_platform(text);
    if platform == config_string("/platform_health_check/generic_platform_label", "电商") {
        config_string("/platform_health_check/default", "")
    } else {
        platform
    }
}

pub(in crate::agent::worker::runner) fn infer_learning_topic(text: &str) -> String {
    let mut topic = match_configured_label_with_dynamic_default(
        text,
        "learning_topic",
        compact_task_text(text),
    );
    for prefix in config_string_list("/label_fields/learning_topic/trim_prefixes") {
        topic = topic.trim_start_matches(&prefix).to_string();
    }
    topic.trim().to_string()
}

pub(in crate::agent::worker::runner) fn infer_learning_level(text: &str) -> String {
    match_configured_label(text, "learning_level")
}

pub(in crate::agent::worker::runner) fn infer_learning_scenario(text: &str) -> String {
    match_configured_label(text, "learning_scenario")
}

fn has_platform_health_check_intent(text: &str) -> bool {
    contains_any(
        text,
        &config_string_list("/platform_health_check/platform_intent_keywords"),
    ) && contains_any(
        text,
        &config_string_list("/platform_health_check/health_intent_keywords"),
    )
}

fn match_configured_label(text: &str, field: &str) -> String {
    let default = config_string(&format!("/label_fields/{field}/default"), "");
    match_configured_label_with_dynamic_default(text, field, default)
}

fn match_configured_label_with_dynamic_default(text: &str, field: &str, default: String) -> String {
    let rule_set = config_string(&format!("/label_fields/{field}/rule_set"), "");
    worker_policies()
        .inference
        .match_label(text, &rule_set)
        .unwrap_or(&default)
        .to_string()
}

fn parse_token_after_label(segment: &str) -> Option<String> {
    let mut chars = segment.char_indices().peekable();
    let mut token_start = None;
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            token_start = Some(index);
            break;
        }
        if matches!(ch, ':' | '：' | '=' | '＝' | ' ' | '\t' | '#' | '号') {
            chars.next();
            continue;
        }
        if index > 12 {
            return None;
        }
        chars.next();
    }

    let start = token_start?;
    let mut end = start;
    for (index, ch) in segment[start ..].char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            end = start + index + ch.len_utf8();
            continue;
        }
        break;
    }
    let token = segment[start .. end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles.iter().any(|needle| {
        let needle = needle.as_ref();
        text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
    })
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("domain", pointer, fallback)
}

fn config_string(pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or("domain", pointer, fallback)
}

fn config_string_list(pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list("domain", pointer)
}
