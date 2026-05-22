use super::{context, domain, metrics, numbers, text::compact_task_text};
use crate::agent::worker::{Task, runner::policy::worker_policies};

pub(in crate::agent::worker::runner) fn query_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = context::first_selected_product(task)
    {
        if let Some(category) = product.category().filter(|value| !value.is_empty()) {
            return format!("{} {}", product.name(), category);
        }
        return product.name().to_string();
    }
    infer_query(text)
}

pub(in crate::agent::worker::runner) fn industry_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = context::first_selected_product(task)
        && let Some(category) = product.category().filter(|value| !value.is_empty())
    {
        return category.to_string();
    }
    domain::infer_category(text)
}

pub(in crate::agent::worker::runner) fn platform_for_task(text: &str, task: &Task) -> String {
    let explicit_platform = infer_platform(text);
    if explicit_platform != config_string("/platform/generic_label", "电商") {
        return explicit_platform;
    }
    context::first_selected_product(task)
        .and_then(|product| product.platform().map(str::to_owned))
        .filter(|platform| !platform.is_empty())
        .unwrap_or(explicit_platform)
}

pub(in crate::agent::worker::runner) fn bypass_cache(text: &str) -> bool {
    contains_any(
        text,
        &config_string_list("/freshness/bypass_cache_keywords"),
    )
}

pub(in crate::agent::worker::runner) fn max_results(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[..start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start..end].parse::<i64>() else {
            continue;
        };
        if value <= 0 {
            continue;
        }
        let before = text[..start]
            .chars()
            .rev()
            .take(config_usize("/max_results/scan_before_chars", 16))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let after = text[end..]
            .chars()
            .take(config_usize("/max_results/scan_after_chars", 16))
            .collect::<String>();

        if is_result_count_context(&before, &after) {
            return Some(value.clamp(
                config_i64("/max_results/min", 1),
                config_i64("/max_results/max", 12),
            ));
        }
    }
    None
}

pub(in crate::agent::worker::runner) fn policy_type(text: &str) -> &'static str {
    match_configured_label(text, "policy_type", "general")
}

pub(in crate::agent::worker::runner) fn review_focus(text: &str) -> &'static str {
    match_configured_label(text, "review_focus", "overall")
}

pub(in crate::agent::worker::runner) fn competitor_focus(text: &str) -> &'static str {
    match_configured_label(text, "competitor_focus", "general")
}

pub(in crate::agent::worker::runner) fn benchmark_metric(text: &str) -> &'static str {
    match_configured_label(text, "benchmark_metric", "general")
}

pub(in crate::agent::worker::runner) fn topic(text: &str) -> &'static str {
    match_configured_label(text, "topic", "general")
}

pub(in crate::agent::worker::runner) fn days(text: &str) -> i64 {
    numbers::infer_period_days(text)
}

fn infer_platform(text: &str) -> String {
    crate::agent::worker::runner::policy::worker_policies()
        .inference
        .match_label(
            text,
            &config_string("/platform/rule_set", "platform.search"),
        )
        .unwrap_or(config_string("/platform/generic_label", "电商").as_str())
        .to_string()
}

fn infer_query(text: &str) -> String {
    let category = domain::infer_category(text);
    if category != config_string("/query/generic_category_label", "电商消费品") {
        return category;
    }
    let keywords = extract_keyword_candidates(text);
    if let Some(keyword) = keywords.first() {
        return keyword.clone();
    }
    compact_task_text(text)
}

fn extract_keyword_candidates(text: &str) -> Vec<String> {
    let min_chars = config_usize("/query/candidate_min_chars", 2);
    let max_chars = config_usize("/query/candidate_max_chars", 24);
    let exclude_keywords = config_string_list("/query/candidate_exclude_keywords");
    let delimiters = split_delimiters("/query/candidate_split_delimiters");

    text.split(|ch| delimiters.contains(&ch))
        .map(str::trim)
        .filter(|value| value.chars().count() >= min_chars && value.chars().count() <= max_chars)
        .filter(|value| !contains_any(value, &exclude_keywords))
        .map(ToOwned::to_owned)
        .collect()
}

fn refers_to_selected_product(text: &str) -> bool {
    contains_any(
        text,
        &config_string_list("/query/selected_product_triggers"),
    )
}

fn is_result_count_context(before: &str, after: &str) -> bool {
    let trimmed_after = after.trim_start();
    let has_after_result_unit = starts_with_any(
        trimmed_after,
        &config_string_list("/max_results/after_result_units"),
    );
    let has_before_limit_word = contains_any(
        before,
        &config_string_list("/max_results/before_limit_keywords"),
    );
    let has_after_short_unit = starts_with_any(
        trimmed_after,
        &config_string_list("/max_results/after_short_units"),
    );

    has_after_result_unit || (has_before_limit_word && has_after_short_unit)
}

fn match_configured_label(text: &str, field: &str, fallback: &'static str) -> &'static str {
    let rule_set = config_string(&format!("/label_fields/{field}/rule_set"), "");
    let default = config_string(&format!("/label_fields/{field}/default"), fallback);
    worker_policies()
        .inference
        .match_label(text, &rule_set)
        .map(str::to_string)
        .unwrap_or(default)
        .leak()
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles.iter().any(|needle| {
        let needle = needle.as_ref();
        text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
    })
}

fn starts_with_any<T: AsRef<str>>(text: &str, prefixes: &[T]) -> bool {
    prefixes
        .iter()
        .any(|prefix| text.starts_with(prefix.as_ref()))
}

fn config_string(pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or("search", pointer, fallback)
}

fn config_string_list(pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list("search", pointer)
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("search", pointer, fallback)
}

fn config_usize(pointer: &str, fallback: usize) -> usize {
    config_i64(pointer, fallback as i64).max(0) as usize
}

fn split_delimiters(pointer: &str) -> Vec<char> {
    config_string_list(pointer)
        .iter()
        .filter_map(|value| {
            let mut chars = value.chars();
            let ch = chars.next()?;
            chars.next().is_none().then_some(ch)
        })
        .collect()
}
