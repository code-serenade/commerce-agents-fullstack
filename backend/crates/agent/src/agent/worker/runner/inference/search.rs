use super::{context, domain, metrics, numbers, text::compact_task_text};
use crate::agent::worker::Task;

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
    if explicit_platform != "电商" {
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
        &[
            "实时",
            "最新",
            "刚刚",
            "今天",
            "联网",
            "公开来源",
            "公开资料",
            "外部资料",
            "真实",
            "真实评价",
            "真实口碑",
            "真实反馈",
            "不要缓存",
            "fresh",
            "live",
            "_needs_fresh_data",
            "bypass_cache",
        ],
    )
}

pub(in crate::agent::worker::runner) fn max_results(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[.. start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start ..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start .. end].parse::<i64>() else {
            continue;
        };
        if value <= 0 {
            continue;
        }
        let before = text[.. start]
            .chars()
            .rev()
            .take(16)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let after = text[end ..].chars().take(16).collect::<String>();

        if is_result_count_context(&before, &after) {
            return Some(value.clamp(1, 12));
        }
    }
    None
}

pub(in crate::agent::worker::runner) fn policy_type(text: &str) -> &'static str {
    match_label(text, "search.policy_type", "general")
}

pub(in crate::agent::worker::runner) fn review_focus(text: &str) -> &'static str {
    match_label(text, "search.review_focus", "overall")
}

pub(in crate::agent::worker::runner) fn competitor_focus(text: &str) -> &'static str {
    match_label(text, "search.competitor_focus", "general")
}

pub(in crate::agent::worker::runner) fn benchmark_metric(text: &str) -> &'static str {
    match_label(text, "search.benchmark_metric", "general")
}

pub(in crate::agent::worker::runner) fn topic(text: &str) -> &'static str {
    match_label(text, "search.topic", "general")
}

pub(in crate::agent::worker::runner) fn days(text: &str) -> i64 {
    numbers::infer_period_days(text)
}

fn infer_platform(text: &str) -> String {
    crate::agent::worker::runner::policy::worker_policies()
        .inference
        .match_label(text, "platform.search")
        .unwrap_or("电商")
        .to_string()
}

fn infer_query(text: &str) -> String {
    let category = domain::infer_category(text);
    if category != "电商消费品" {
        return category;
    }
    let keywords = extract_keyword_candidates(text);
    if let Some(keyword) = keywords.first() {
        return keyword.clone();
    }
    compact_task_text(text)
}

fn extract_keyword_candidates(text: &str) -> Vec<String> {
    text.split(['，', ',', '；', ';', '。', '.', '\n'])
        .map(str::trim)
        .filter(|value| value.chars().count() >= 2 && value.chars().count() <= 24)
        .filter(|value| {
            !contains_any(
                value,
                &[
                    "帮我", "请", "搜索", "查询", "分析", "最新", "实时", "行业", "数据",
                ],
            )
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn refers_to_selected_product(text: &str) -> bool {
    contains_any(
        text,
        &[
            "这个商品",
            "这个产品",
            "该商品",
            "该产品",
            "本品",
            "当前商品",
            "所选商品",
            "选中商品",
            "这个SKU",
            "该SKU",
            "this product",
            "selected product",
        ],
    )
}

fn is_result_count_context(before: &str, after: &str) -> bool {
    let trimmed_after = after.trim_start();
    let has_after_result_unit = [
        "条来源",
        "个来源",
        "条结果",
        "个结果",
        "条公开资料",
        "个公开资料",
        "条证据",
        "个证据",
        "篇资料",
        "篇来源",
        "sources",
        "source",
        "results",
        "result",
        "links",
        "items",
    ]
    .iter()
    .any(|unit| trimmed_after.starts_with(unit));
    let has_before_limit_word = contains_any(
        before,
        &[
            "最多", "至多", "只要", "只看", "返回", "给我", "列出", "展示", "查", "搜", "top",
            "limit", "max", "maximum",
        ],
    );
    let has_after_short_unit = trimmed_after.starts_with('条')
        || trimmed_after.starts_with('个')
        || trimmed_after.starts_with("篇");

    has_after_result_unit || (has_before_limit_word && has_after_short_unit)
}

fn match_label(text: &str, rule_set: &str, default: &'static str) -> &'static str {
    crate::agent::worker::runner::policy::worker_policies()
        .inference
        .match_label(text, rule_set)
        .unwrap_or(default)
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
