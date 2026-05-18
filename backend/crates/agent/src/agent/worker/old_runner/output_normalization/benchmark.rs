use super::*;
use super::normalization_policy::{contains_any, policy as normalization_policy};

pub(super) fn sanitize_unsupported_benchmark_claims(output: &mut RoleWorkOutput) -> bool {
    let mut removed = false;
    if is_unsupported_benchmark_claim(&output.summary) {
        output.summary = normalization_policy()
            .unsupported_benchmark()
            .summary_replacement()
            .to_string();
        removed = true;
    }
    removed |= retain_supported_benchmark_items(&mut output.findings);
    removed |= retain_supported_benchmark_items(&mut output.recommendations);
    removed |= retain_supported_benchmark_items(&mut output.risks);
    removed |= retain_supported_benchmark_items(&mut output.open_questions);
    removed |= retain_supported_benchmark_items(&mut output.evidence);
    removed
}

fn retain_supported_benchmark_items(items: &mut Vec<String>) -> bool {
    let original_len = items.len();
    items.retain(|item| !is_unsupported_benchmark_claim(item));
    original_len != items.len()
}

fn is_unsupported_benchmark_claim(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return false;
    }
    let policy = normalization_policy().unsupported_benchmark();
    if contains_any(text, policy.allowed_source_markers()) {
        return false;
    }

    let has_assertive_comparison = contains_any(text, policy.assertive_comparison_terms());
    if has_assertive_comparison {
        return true;
    }

    let has_benchmark = contains_any(text, policy.benchmark_terms());
    if !has_benchmark {
        return false;
    }

    let asks_for_benchmark = contains_any(text, policy.benchmark_request_terms());
    if asks_for_benchmark && !contains_benchmark_number_claim(text) {
        return false;
    }
    if contains_any(text, policy.assertive_benchmark_terms()) {
        return true;
    }
    contains_benchmark_number_claim(text)
}

fn contains_benchmark_number_claim(value: &str) -> bool {
    normalization_policy()
        .unsupported_benchmark()
        .benchmark_number_terms()
        .iter()
        .any(|term| {
        value.find(term).is_some_and(|index| {
            let after = &value[index ..];
            let before = &value[.. index];
            segment_after_contains_percent_or_range(after)
                || segment_before_contains_percent_or_range(before)
        })
    })
}

fn segment_after_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .split(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn segment_before_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .rsplit(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn contains_percent_or_range(value: &str) -> bool {
    let mut previous_was_digit = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            previous_was_digit = true;
            continue;
        }
        if previous_was_digit && matches!(ch, '%' | '％' | '-' | '~' | '—' | '至') {
            return true;
        }
        if !matches!(ch, '.' | ' ' | '\t') {
            previous_was_digit = false;
        }
    }
    false
}
